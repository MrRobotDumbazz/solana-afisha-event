package store

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"afisha/api/internal/model"
)

const schema = `
CREATE TABLE IF NOT EXISTS events (
  pubkey TEXT PRIMARY KEY,
  organizer TEXT NOT NULL,
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  venue TEXT NOT NULL,
  city TEXT NOT NULL,
  image_uri TEXT NOT NULL,
  starts_at BIGINT NOT NULL,
  ends_at BIGINT NOT NULL,
  ticket_price_lamports BIGINT NOT NULL,
  capacity INT NOT NULL,
  tickets_sold INT NOT NULL,
  hot_sale BOOLEAN NOT NULL,
  status SMALLINT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS events_organizer_slug ON events (organizer, slug);

CREATE TABLE IF NOT EXISTS tickets (
  pubkey TEXT PRIMARY KEY,
  event_pubkey TEXT NOT NULL,
  buyer TEXT NOT NULL,
  mint TEXT NOT NULL UNIQUE,
  status SMALLINT NOT NULL,
  checked_in_at BIGINT NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS tickets_buyer ON tickets (buyer);
CREATE INDEX IF NOT EXISTS tickets_event ON tickets (event_pubkey);

CREATE TABLE IF NOT EXISTS sales (
  event_pubkey TEXT PRIMARY KEY REFERENCES events (pubkey) ON DELETE CASCADE,
  registration_start BIGINT NOT NULL,
  registration_end BIGINT NOT NULL,
  reveal_at BIGINT NOT NULL,
  claim_start BIGINT NOT NULL,
  round_duration_secs BIGINT NOT NULL,
  stake_lamports BIGINT NOT NULL,
  window_size INT NOT NULL,
  total_entries INT NOT NULL,
  randomness BIGINT NOT NULL,
  settled BOOLEAN NOT NULL,
  claimed INT NOT NULL,
  settled_count INT NOT NULL,
  forfeited_count INT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS queue_entries (
  pubkey TEXT PRIMARY KEY,
  event_pubkey TEXT NOT NULL,
  buyer TEXT NOT NULL,
  position INT NOT NULL,
  stake_lamports BIGINT NOT NULL,
  status SMALLINT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS queue_event_buyer ON queue_entries (event_pubkey, buyer);
CREATE INDEX IF NOT EXISTS queue_event ON queue_entries (event_pubkey);
`

type Store struct {
	pool *pgxpool.Pool
}

func Open(ctx context.Context, databaseURL string) (*Store, error) {
	cfg, err := pgxpool.ParseConfig(databaseURL)
	if err != nil {
		return nil, fmt.Errorf("parse database url: %w", err)
	}
	cfg.MaxConns = 4
	pool, err := pgxpool.NewWithConfig(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("connect postgres: %w", err)
	}
	if _, err := pool.Exec(ctx, schema); err != nil {
		pool.Close()
		return nil, fmt.Errorf("apply schema: %w", err)
	}
	return &Store{pool: pool}, nil
}

func (s *Store) Close() {
	s.pool.Close()
}

func (s *Store) Pool() *pgxpool.Pool {
	return s.pool
}

func (s *Store) UpsertEvent(ctx context.Context, e *model.Event) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO events (pubkey, organizer, slug, title, description, venue, city, image_uri,
			starts_at, ends_at, ticket_price_lamports, capacity, tickets_sold, hot_sale, status, updated_at)
		VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15, now())
		ON CONFLICT (pubkey) DO UPDATE SET
			organizer=EXCLUDED.organizer, slug=EXCLUDED.slug, title=EXCLUDED.title,
			description=EXCLUDED.description, venue=EXCLUDED.venue, city=EXCLUDED.city,
			image_uri=EXCLUDED.image_uri, starts_at=EXCLUDED.starts_at, ends_at=EXCLUDED.ends_at,
			ticket_price_lamports=EXCLUDED.ticket_price_lamports, capacity=EXCLUDED.capacity,
			tickets_sold=EXCLUDED.tickets_sold, hot_sale=EXCLUDED.hot_sale,
			status=EXCLUDED.status, updated_at=now()`,
		e.Pubkey, e.Organizer, e.Slug, e.Title, e.Description, e.Venue, e.City, e.ImageURI,
		e.StartsAt, e.EndsAt, e.TicketPriceLamport, e.Capacity, e.TicketsSold, e.HotSale, e.Status)
	return err
}

func (s *Store) UpsertTicket(ctx context.Context, t *model.Ticket) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO tickets (pubkey, event_pubkey, buyer, mint, status, checked_in_at, updated_at)
		VALUES ($1,$2,$3,$4,$5,$6, now())
		ON CONFLICT (pubkey) DO UPDATE SET
			event_pubkey=EXCLUDED.event_pubkey, buyer=EXCLUDED.buyer, mint=EXCLUDED.mint,
			status=EXCLUDED.status, checked_in_at=EXCLUDED.checked_in_at, updated_at=now()`,
		t.Pubkey, t.EventPubkey, t.Buyer, t.Mint, t.Status, t.CheckedInAt)
	return err
}

func (s *Store) UpsertSale(ctx context.Context, sa *model.Sale) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO sales (event_pubkey, registration_start, registration_end, reveal_at, claim_start,
			round_duration_secs, stake_lamports, window_size, total_entries, randomness, settled,
			claimed, settled_count, forfeited_count, updated_at)
		VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14, now())
		ON CONFLICT (event_pubkey) DO UPDATE SET
			registration_start=EXCLUDED.registration_start, registration_end=EXCLUDED.registration_end,
			reveal_at=EXCLUDED.reveal_at, claim_start=EXCLUDED.claim_start,
			round_duration_secs=EXCLUDED.round_duration_secs, stake_lamports=EXCLUDED.stake_lamports,
			window_size=EXCLUDED.window_size, total_entries=EXCLUDED.total_entries,
			randomness=EXCLUDED.randomness, settled=EXCLUDED.settled, claimed=EXCLUDED.claimed,
			settled_count=EXCLUDED.settled_count, forfeited_count=EXCLUDED.forfeited_count, updated_at=now()`,
		sa.EventPubkey, sa.RegistrationStart, sa.RegistrationEnd, sa.RevealAt, sa.ClaimStart,
		sa.RoundDurationSecs, sa.StakeLamports, sa.WindowSize, sa.TotalEntries,
		int64(sa.Randomness), sa.Settled, sa.Claimed, sa.SettledCount, sa.ForfeitedCount)
	return err
}

func (s *Store) UpsertQueueEntry(ctx context.Context, q *model.QueueEntry) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO queue_entries (pubkey, event_pubkey, buyer, position, stake_lamports, status, updated_at)
		VALUES ($1,$2,$3,$4,$5,$6, now())
		ON CONFLICT (pubkey) DO UPDATE SET
			event_pubkey=EXCLUDED.event_pubkey, buyer=EXCLUDED.buyer, position=EXCLUDED.position,
			stake_lamports=EXCLUDED.stake_lamports, status=EXCLUDED.status, updated_at=now()`,
		q.Pubkey, q.EventPubkey, q.Buyer, q.Position, q.StakeLamports, q.Status)
	return err
}

func (s *Store) DeleteAccount(ctx context.Context, pubkey string) error {
	for _, q := range []string{
		`DELETE FROM tickets WHERE pubkey = $1`,
		`DELETE FROM queue_entries WHERE pubkey = $1`,
		`DELETE FROM sales WHERE event_pubkey = $1`,
		`DELETE FROM events WHERE pubkey = $1`,
	} {
		if _, err := s.pool.Exec(ctx, q, pubkey); err != nil {
			return err
		}
	}
	return nil
}

type EventFilter struct {
	City      string
	Query     string
	Status    string
	Organizer string
	Upcoming  bool
	Limit     int
	Offset    int
}

func (s *Store) ListEvents(ctx context.Context, f EventFilter) ([]model.Event, int, error) {
	where := "WHERE TRUE"
	args := []any{}
	n := 1
	if f.City != "" {
		where += fmt.Sprintf(" AND lower(city) = lower($%d)", n)
		args = append(args, f.City)
		n++
	}
	if f.Query != "" {
		where += fmt.Sprintf(" AND (title ILIKE '%%' || $%d || '%%' OR description ILIKE '%%' || $%d || '%%' OR venue ILIKE '%%' || $%d || '%%')", n, n, n)
		args = append(args, f.Query)
		n++
	}
	if f.Organizer != "" {
		where += fmt.Sprintf(" AND organizer = $%d", n)
		args = append(args, f.Organizer)
		n++
	}
	if f.Status != "" {
		where += fmt.Sprintf(" AND status = $%d", n)
		args = append(args, statusFromText(f.Status))
		n++
	}
	if f.Upcoming {
		where += fmt.Sprintf(" AND ends_at > $%d", n)
		args = append(args, time.Now().Unix())
		n++
	}

	var total int
	if err := s.pool.QueryRow(ctx, "SELECT count(*) FROM events "+where, args...).Scan(&total); err != nil {
		return nil, 0, err
	}

	limit, offset := f.Limit, f.Offset
	if limit <= 0 || limit > 100 {
		limit = 20
	}
	if offset < 0 {
		offset = 0
	}
	q := fmt.Sprintf(`SELECT pubkey, organizer, slug, title, description, venue, city, image_uri,
			starts_at, ends_at, ticket_price_lamports, capacity, tickets_sold, hot_sale, status,
			EXISTS(SELECT 1 FROM sales WHERE sales.event_pubkey = events.pubkey)
		FROM events %s ORDER BY starts_at ASC LIMIT %d OFFSET %d`,
		where, limit, offset)

	rows, err := s.pool.Query(ctx, q, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	type row struct {
		model.Event
		SaleConfigured bool
	}
	var out []model.Event
	for rows.Next() {
		var r row
		if err := rows.Scan(&r.Pubkey, &r.Organizer, &r.Slug, &r.Title, &r.Description,
			&r.Venue, &r.City, &r.ImageURI, &r.StartsAt, &r.EndsAt, &r.TicketPriceLamport,
			&r.Capacity, &r.TicketsSold, &r.HotSale, &r.Status, &r.SaleConfigured); err != nil {
			return nil, 0, err
		}
		if r.SaleConfigured {
			r.Event.SaleConfigured = true
		}
		out = append(out, r.Event)
	}
	return out, total, rows.Err()
}

func (s *Store) GetEvent(ctx context.Context, pubkey string) (*model.Event, error) {
	var e model.Event
	err := s.pool.QueryRow(ctx, `
		SELECT pubkey, organizer, slug, title, description, venue, city, image_uri,
			starts_at, ends_at, ticket_price_lamports, capacity, tickets_sold, hot_sale, status
		FROM events WHERE pubkey = $1`, pubkey).
		Scan(&e.Pubkey, &e.Organizer, &e.Slug, &e.Title, &e.Description, &e.Venue, &e.City,
			&e.ImageURI, &e.StartsAt, &e.EndsAt, &e.TicketPriceLamport, &e.Capacity,
			&e.TicketsSold, &e.HotSale, &e.Status)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &e, nil
}

func (s *Store) GetSale(ctx context.Context, eventPubkey string) (*model.Sale, error) {
	var sa model.Sale
	var randomness int64
	err := s.pool.QueryRow(ctx, `
		SELECT event_pubkey, registration_start, registration_end, reveal_at, claim_start,
			round_duration_secs, stake_lamports, window_size, total_entries, randomness, settled,
			claimed, settled_count, forfeited_count
		FROM sales WHERE event_pubkey = $1`, eventPubkey).
		Scan(&sa.EventPubkey, &sa.RegistrationStart, &sa.RegistrationEnd, &sa.RevealAt,
			&sa.ClaimStart, &sa.RoundDurationSecs, &sa.StakeLamports, &sa.WindowSize,
			&sa.TotalEntries, &randomness, &sa.Settled, &sa.Claimed, &sa.SettledCount,
			&sa.ForfeitedCount)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	sa.Randomness = uint64(randomness)
	return &sa, nil
}

func (s *Store) ListTicketsForEvent(ctx context.Context, eventPubkey string) ([]model.Ticket, error) {
	rows, err := s.pool.Query(ctx, `
		SELECT pubkey, event_pubkey, buyer, mint, status, checked_in_at
		FROM tickets WHERE event_pubkey = $1 ORDER BY pubkey`, eventPubkey)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	return scanTickets(rows)
}

func (s *Store) ListTicketsForWallet(ctx context.Context, wallet string) ([]model.WalletTicket, error) {
	rows, err := s.pool.Query(ctx, `
		SELECT t.pubkey, t.event_pubkey, t.buyer, t.mint, t.status, t.checked_in_at,
			e.title, e.starts_at, e.ends_at, e.city, e.status
		FROM tickets t JOIN events e ON e.pubkey = t.event_pubkey
		WHERE t.buyer = $1 ORDER BY e.starts_at DESC`, wallet)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var out []model.WalletTicket
	for rows.Next() {
		var w model.WalletTicket
		var status int
		if err := rows.Scan(&w.Pubkey, &w.EventPubkey, &w.Buyer, &w.Mint, &w.Status,
			&w.CheckedInAt, &w.EventTitle, &w.EventStarts, &w.EventEnds, &w.EventCity,
			&status); err != nil {
			return nil, err
		}
		w.EventStatus = model.EventStatus(status).String()
		out = append(out, w)
	}
	return out, rows.Err()
}

func (s *Store) GetTicketByMint(ctx context.Context, mint string) (*model.WalletTicket, error) {
	var w model.WalletTicket
	var status int
	err := s.pool.QueryRow(ctx, `
		SELECT t.pubkey, t.event_pubkey, t.buyer, t.mint, t.status, t.checked_in_at,
			e.title, e.starts_at, e.ends_at, e.city, e.status
		FROM tickets t JOIN events e ON e.pubkey = t.event_pubkey
		WHERE t.mint = $1`, mint).
		Scan(&w.Pubkey, &w.EventPubkey, &w.Buyer, &w.Mint, &w.Status, &w.CheckedInAt,
			&w.EventTitle, &w.EventStarts, &w.EventEnds, &w.EventCity, &status)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	w.EventStatus = model.EventStatus(status).String()
	return &w, nil
}

func (s *Store) GetQueueEntry(ctx context.Context, eventPubkey, buyer string) (*model.QueueEntry, error) {
	var q model.QueueEntry
	err := s.pool.QueryRow(ctx, `
		SELECT pubkey, event_pubkey, buyer, position, stake_lamports, status
		FROM queue_entries WHERE event_pubkey = $1 AND buyer = $2`, eventPubkey, buyer).
		Scan(&q.Pubkey, &q.EventPubkey, &q.Buyer, &q.Position, &q.StakeLamports, &q.Status)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &q, nil
}

func (s *Store) ListQueue(ctx context.Context, eventPubkey string, limit int) ([]model.QueueEntry, error) {
	if limit <= 0 || limit > 1000 {
		limit = 100
	}
	rows, err := s.pool.Query(ctx, `
		SELECT pubkey, event_pubkey, buyer, position, stake_lamports, status
		FROM queue_entries WHERE event_pubkey = $1 ORDER BY position LIMIT $2`, eventPubkey, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var out []model.QueueEntry
	for rows.Next() {
		var q model.QueueEntry
		if err := rows.Scan(&q.Pubkey, &q.EventPubkey, &q.Buyer, &q.Position, &q.StakeLamports, &q.Status); err != nil {
			return nil, err
		}
		out = append(out, q)
	}
	return out, rows.Err()
}

func scanTickets(rows pgx.Rows) ([]model.Ticket, error) {
	var out []model.Ticket
	for rows.Next() {
		var t model.Ticket
		if err := rows.Scan(&t.Pubkey, &t.EventPubkey, &t.Buyer, &t.Mint, &t.Status, &t.CheckedInAt); err != nil {
			return nil, err
		}
		out = append(out, t)
	}
	return out, rows.Err()
}

func statusFromText(s string) int {
	if model.EventStatusActive.String() == s {
		return 0
	}
	return 1
}
