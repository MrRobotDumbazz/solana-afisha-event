package indexer

import (
	"context"
	"log/slog"
	"time"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/gagliardetto/solana-go/rpc/ws"

	"afisha/api/internal/decode"
	"afisha/api/internal/store"
)

type Indexer struct {
	rpcClient *rpc.Client
	wsClient  *ws.Client
	programID solana.PublicKey
	store     *store.Store
	logger    *slog.Logger
}

func New(rpcURL, wsURL, programID string, st *store.Store, logger *slog.Logger) (*Indexer, error) {
	rpcClient := rpc.New(rpcURL)
	wsClient, err := ws.Connect(context.Background(), wsURL)
	if err != nil {
		return nil, err
	}
	pid, err := solana.PublicKeyFromBase58(programID)
	if err != nil {
		return nil, err
	}
	return &Indexer{
		rpcClient: rpcClient,
		wsClient:  wsClient,
		programID: pid,
		store:     st,
		logger:    logger,
	}, nil
}

func (ix *Indexer) Run(ctx context.Context, resyncEvery time.Duration) error {
	if err := ix.SyncAll(ctx); err != nil {
		ix.logger.Error("initial sync failed", "err", err)
	}

	sub, err := ix.wsClient.LogsSubscribeMentions(ix.programID, "")
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()
	ix.logger.Info("subscribed to program logs", "program", ix.programID.String())

	logsCtx, cancel := context.WithCancel(ctx)
	defer cancel()
	go func() {
		for {
			result, err := sub.Recv(logsCtx)
			if err != nil {
				if logsCtx.Err() == nil {
					ix.logger.Warn("log subscription error", "err", err)
				}
				return
			}
			if result != nil {
				sig := result.Value.Signature
				go ix.handleSignature(context.Background(), sig)
			}
		}
	}()

	ticker := time.NewTicker(resyncEvery)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			if err := ix.SyncAll(ctx); err != nil {
				ix.logger.Error("periodic resync failed", "err", err)
			}
		}
	}
}

func (ix *Indexer) SyncAll(ctx context.Context) error {
	start := time.Now()
	counts := map[string]int{}

	evs, err := ix.fetchAccounts(ctx, decode.DiscEvent)
	if err != nil {
		return err
	}
	for _, data := range evs {
		e, err := decode.Event(data.data)
		if err != nil {
			ix.logger.Warn("bad event data", "pubkey", data.pubkey, "err", err)
			continue
		}
		e.Pubkey = data.pubkey
		if err := ix.store.UpsertEvent(ctx, e); err != nil {
			return err
		}
		counts["events"]++
	}

	tks, err := ix.fetchAccounts(ctx, decode.DiscTicket)
	if err != nil {
		return err
	}
	for _, data := range tks {
		t, err := decode.Ticket(data.data)
		if err != nil {
			ix.logger.Warn("bad ticket data", "pubkey", data.pubkey, "err", err)
			continue
		}
		t.Pubkey = data.pubkey
		if err := ix.store.UpsertTicket(ctx, t); err != nil {
			ix.logger.Warn("ticket upsert skipped", "pubkey", data.pubkey, "err", err)
		}
		counts["tickets"]++
	}

	sales, err := ix.fetchAccounts(ctx, decode.DiscSaleState)
	if err != nil {
		return err
	}
	for _, data := range sales {
		sa, err := decode.SaleState(data.data)
		if err != nil {
			ix.logger.Warn("bad sale data", "pubkey", data.pubkey, "err", err)
			continue
		}
		if err := ix.store.UpsertSale(ctx, sa); err != nil {
			ix.logger.Warn("sale upsert skipped", "pubkey", data.pubkey, "err", err)
		}
		counts["sales"]++
	}

	qs, err := ix.fetchAccounts(ctx, decode.DiscQueueEntry)
	if err != nil {
		return err
	}
	for _, data := range qs {
		q, err := decode.QueueEntry(data.data)
		if err != nil {
			ix.logger.Warn("bad queue data", "pubkey", data.pubkey, "err", err)
			continue
		}
		q.Pubkey = data.pubkey
		if err := ix.store.UpsertQueueEntry(ctx, q); err != nil {
			ix.logger.Warn("queue upsert skipped", "pubkey", data.pubkey, "err", err)
		}
		counts["queue"]++
	}

	ix.logger.Info("sync complete",
		"events", counts["events"],
		"tickets", counts["tickets"],
		"sales", counts["sales"],
		"queue", counts["queue"],
		"took", time.Since(start).Round(time.Millisecond).String())
	return nil
}

type fetched struct {
	pubkey string
	data   []byte
}

func (ix *Indexer) fetchAccounts(ctx context.Context, disc []byte) ([]fetched, error) {
	accounts, err := ix.rpcClient.GetProgramAccountsWithOpts(
		ctx,
		ix.programID,
		&rpc.GetProgramAccountsOpts{
			Filters: []rpc.RPCFilter{
				{
					Memcmp: &rpc.RPCFilterMemcmp{
						Offset: 0,
						Bytes:  solana.Base58(disc),
					},
				},
			},
			Encoding: solana.EncodingBase64,
		})
	if err != nil {
		return nil, err
	}
	out := make([]fetched, 0, len(accounts))
	for _, acc := range accounts {
		if acc.Account == nil {
			continue
		}
		out = append(out, fetched{pubkey: acc.Pubkey.String(), data: acc.Account.Data.GetBinary()})
	}
	return out, nil
}

func (ix *Indexer) handleSignature(ctx context.Context, sig solana.Signature) {
	// Give the RPC node a moment to make the transaction available.
	select {
	case <-time.After(2 * time.Second):
	case <-ctx.Done():
		return
	}

	tx, err := ix.rpcClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
		Encoding:                       solana.EncodingBase64,
		Commitment:                     rpc.CommitmentConfirmed,
		MaxSupportedTransactionVersion: &maxTxVersionValue,
	})
	if err != nil {
		ix.logger.Warn("getTransaction failed", "sig", sig.String(), "err", err)
		return
	}
	if tx == nil || tx.Transaction == nil {
		return
	}

	parsed, err := solana.TransactionFromBytes(tx.Transaction.GetBinary())
	if err != nil {
		ix.logger.Warn("tx decode failed", "sig", sig.String(), "err", err)
		return
	}

	seen := map[string]bool{}
	var uniq []solana.PublicKey
	keys := append([]solana.PublicKey{}, parsed.Message.AccountKeys...)
	if tx.Meta != nil {
		keys = append(keys, tx.Meta.LoadedAddresses.Writable...)
		keys = append(keys, tx.Meta.LoadedAddresses.ReadOnly...)
	}
	for _, k := range keys {
		s := k.String()
		if !seen[s] {
			seen[s] = true
			uniq = append(uniq, k)
		}
	}

	ix.refreshAccounts(ctx, uniq)
}

var maxTxVersionValue = uint64(0)

func (ix *Indexer) refreshAccounts(ctx context.Context, keys []solana.PublicKey) {
	const chunk = 100
	for i := 0; i < len(keys); i += chunk {
		end := i + chunk
		if end > len(keys) {
			end = len(keys)
		}
		infos, err := ix.rpcClient.GetMultipleAccounts(ctx, keys[i:end]...)
		if err != nil {
			ix.logger.Warn("getMultipleAccounts failed", "err", err)
			continue
		}
		if infos == nil || infos.Value == nil {
			continue
		}
		for j, info := range infos.Value {
			pub := keys[i+j].String()
			if info == nil {
				if err := ix.store.DeleteAccount(ctx, pub); err != nil {
					ix.logger.Warn("delete failed", "pubkey", pub, "err", err)
				}
				continue
			}
			data := info.Data.GetBinary()
			switch decode.Classify(data) {
			case decode.KindEvent:
				e, err := decode.Event(data)
				if err == nil {
					e.Pubkey = pub
					if err := ix.store.UpsertEvent(ctx, e); err != nil {
						ix.logger.Warn("event upsert failed", "pubkey", pub, "err", err)
					}
				}
			case decode.KindTicket:
				t, err := decode.Ticket(data)
				if err == nil {
					t.Pubkey = pub
					if err := ix.store.UpsertTicket(ctx, t); err != nil {
						ix.logger.Warn("ticket upsert failed", "pubkey", pub, "err", err)
					}
				}
			case decode.KindSaleState:
				sa, err := decode.SaleState(data)
				if err == nil {
					if err := ix.store.UpsertSale(ctx, sa); err != nil {
						ix.logger.Warn("sale upsert failed", "pubkey", pub, "err", err)
					}
				}
			case decode.KindQueueEntry:
				q, err := decode.QueueEntry(data)
				if err == nil {
					q.Pubkey = pub
					if err := ix.store.UpsertQueueEntry(ctx, q); err != nil {
						ix.logger.Warn("queue upsert failed", "pubkey", pub, "err", err)
					}
				}
			}
		}
	}
}
