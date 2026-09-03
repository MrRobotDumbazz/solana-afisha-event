package main

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"afisha/api/internal/api"
	"afisha/api/internal/auth"
	"afisha/api/internal/indexer"
	"afisha/api/internal/store"
)

type config struct {
	Port         string
	DatabaseURL  string
	RPCURL       string
	WSURL        string
	ProgramID    string
	ResyncPeriod time.Duration
}

func loadConfig() config {
	return config{
		Port:         envOr("PORT", "8080"),
		DatabaseURL:  envOr("DATABASE_URL", "postgres://postgres:postgres@127.0.0.1:5432/afisha?sslmode=disable"),
		RPCURL:       envOr("RPC_URL", "https://api.devnet.solana.com"),
		WSURL:        envOr("WS_URL", defaultWS(os.Getenv("RPC_URL"))),
		ProgramID:    envOr("PROGRAM_ID", "7J6VC2HsTxBCBMc94FbcfT2NcN2bmSK5nhjejeuL4g8Y"),
		ResyncPeriod: 10 * time.Minute,
	}
}

func defaultWS(rpcURL string) string {
	if rpcURL == "" {
		return "wss://api.devnet.solana.com"
	}
	ws := strings.Replace(rpcURL, "https://", "wss://", 1)
	ws = strings.Replace(ws, "http://", "ws://", 1)
	return ws
}

func envOr(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))
	cfg := loadConfig()

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	st, err := store.Open(ctx, cfg.DatabaseURL)
	if err != nil {
		logger.Error("store open failed", "err", err)
		os.Exit(1)
	}
	defer st.Close()
	logger.Info("postgres ready", "url", redact(cfg.DatabaseURL))

	runIndexer := os.Getenv("DISABLE_INDEXER") == ""
	if runIndexer {
		ix, err := indexer.New(cfg.RPCURL, cfg.WSURL, cfg.ProgramID, st, logger)
		if err != nil {
			logger.Error("indexer init failed", "err", err)
			os.Exit(1)
		}
		go func() {
			if err := ix.Run(ctx, cfg.ResyncPeriod); err != nil && ctx.Err() == nil {
				logger.Error("indexer stopped", "err", err)
			}
		}()
		logger.Info("indexer started", "rpc", cfg.RPCURL, "ws", cfg.WSURL, "program", cfg.ProgramID)
	} else {
		logger.Info("indexer disabled")
	}

	srv := api.New(st, logger)
	httpSrv := &http.Server{
		Addr:              ":" + cfg.Port,
		Handler:           srv.Routes(auth.NewServer(auth.NewManager(logger), logger).Routes()),
		ReadHeaderTimeout: 5 * time.Second,
	}

	go func() {
		logger.Info("api listening", "port", cfg.Port)
		if err := httpSrv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			logger.Error("listen failed", "err", err)
			os.Exit(1)
		}
	}()

	<-ctx.Done()
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	_ = httpSrv.Shutdown(shutdownCtx)
	logger.Info("api stopped")
}

func redact(databaseURL string) string {
	if at := strings.Index(databaseURL, "@"); at > 0 {
		if slash := strings.Index(databaseURL, "//"); slash >= 0 && slash+2 < at {
			return databaseURL[:slash+2] + "***" + databaseURL[at:]
		}
	}
	return databaseURL
}
