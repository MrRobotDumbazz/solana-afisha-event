package api

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	"afisha/api/internal/model"
	"afisha/api/internal/store"
)

type Server struct {
	store  *store.Store
	logger *slog.Logger
}

func New(st *store.Store, logger *slog.Logger) *Server {
	return &Server{store: st, logger: logger}
}

func (s *Server) Routes() *http.ServeMux {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", s.handleHealth)
	mux.HandleFunc("GET /api/v1/events", s.handleEvents)
	mux.HandleFunc("GET /api/v1/events/{pubkey}", s.handleEvent)
	mux.HandleFunc("GET /api/v1/events/{pubkey}/tickets", s.handleEventTickets)
	mux.HandleFunc("GET /api/v1/events/{pubkey}/queue", s.handleEventQueue)
	mux.HandleFunc("GET /api/v1/wallets/{pubkey}/tickets", s.handleWalletTickets)
	mux.HandleFunc("GET /api/v1/tickets/mint/{mint}", s.handleTicketByMint)
	return mux
}

func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (s *Server) handleEvents(w http.ResponseWriter, r *http.Request) {
	q := r.URL.Query()
	limit, _ := strconv.Atoi(q.Get("limit"))
	offset, _ := strconv.Atoi(q.Get("offset"))

	events, total, err := s.store.ListEvents(r.Context(), store.EventFilter{
		City:      q.Get("city"),
		Query:     q.Get("q"),
		Status:    q.Get("status"),
		Organizer: q.Get("organizer"),
		Upcoming:  q.Get("upcoming") == "true",
		Limit:     limit,
		Offset:    offset,
	})
	if err != nil {
		s.logger.Error("list events", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}

	type item struct {
		model.Event
		StatusText  string `json:"status"`
		TicketsLeft uint32 `json:"tickets_left"`
	}
	items := make([]item, 0, len(events))
	for _, e := range events {
		items = append(items, item{
			Event:       e,
			StatusText:  model.EventStatus(e.Status).String(),
			TicketsLeft: e.Capacity - e.TicketsSold,
		})
	}
	writeJSON(w, http.StatusOK, map[string]any{"events": items, "total": total})
}

func (s *Server) handleEvent(w http.ResponseWriter, r *http.Request) {
	pubkey := r.PathValue("pubkey")
	event, err := s.store.GetEvent(r.Context(), pubkey)
	if err != nil {
		s.logger.Error("get event", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	if event == nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "event not found"})
		return
	}

	details := model.EventDetails{
		Event:       *event,
		StatusText:  model.EventStatus(event.Status).String(),
		TicketsLeft: event.Capacity - event.TicketsSold,
	}
	if event.HotSale {
		sa, err := s.store.GetSale(r.Context(), pubkey)
		if err != nil {
			s.logger.Error("get sale", "err", err)
		} else {
			details.Sale = sa
		}
	}
	writeJSON(w, http.StatusOK, details)
}

func (s *Server) handleEventTickets(w http.ResponseWriter, r *http.Request) {
	pubkey := r.PathValue("pubkey")
	event, err := s.store.GetEvent(r.Context(), pubkey)
	if err != nil {
		s.logger.Error("get event", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	if event == nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "event not found"})
		return
	}

	type item struct {
		model.Ticket
		StatusText string `json:"status"`
	}
	tickets, err := s.store.ListTicketsForEvent(r.Context(), pubkey)
	if err != nil {
		s.logger.Error("list tickets", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	items := make([]item, 0, len(tickets))
	for _, t := range tickets {
		items = append(items, item{Ticket: t, StatusText: model.TicketStatus(t.Status).String()})
	}
	writeJSON(w, http.StatusOK, map[string]any{"tickets": items})
}

func (s *Server) handleEventQueue(w http.ResponseWriter, r *http.Request) {
	pubkey := r.PathValue("pubkey")
	event, err := s.store.GetEvent(r.Context(), pubkey)
	if err != nil {
		s.logger.Error("get event", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	if event == nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "event not found"})
		return
	}

	sa, err := s.store.GetSale(r.Context(), pubkey)
	if err != nil {
		s.logger.Error("get sale", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	if sa == nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "no queue for this event"})
		return
	}

	view := model.QueueView{Sale: *sa}
	now := time.Now().Unix()
	view.Phase = phase(sa, now)
	view.Pending = sa.TotalEntries - sa.Claimed - sa.SettledCount - sa.ForfeitedCount

	if sa.Settled && now >= sa.ClaimStart && sa.RoundDurationSecs > 0 {
		view.CurrentRound = (now - sa.ClaimStart) / sa.RoundDurationSecs
		from := uint32(view.CurrentRound) * sa.WindowSize
		to := from + sa.WindowSize
		if to > sa.TotalEntries {
			to = sa.TotalEntries
		}
		if from > sa.TotalEntries {
			from = sa.TotalEntries
		}
		view.RoundServingFrom = from
		view.RoundServingTo = to
	}

	if buyer := r.URL.Query().Get("buyer"); buyer != "" {
		entry, err := s.store.GetQueueEntry(r.Context(), pubkey, buyer)
		if err != nil {
			s.logger.Error("get entry", "err", err)
		} else if entry != nil {
			type withEntry struct {
				model.QueueView
				MyEntry *queueEntryView `json:"my_entry,omitempty"`
			}
			out := withEntry{QueueView: view, MyEntry: entryView(sa, entry)}
			writeJSON(w, http.StatusOK, out)
			return
		}
	}

	writeJSON(w, http.StatusOK, view)
}

type queueEntryView struct {
	Pubkey        string `json:"pubkey"`
	Buyer         string `json:"buyer"`
	Position      uint32 `json:"position"`
	EffectivePos  uint32 `json:"effective_position"`
	Round         int64  `json:"round"`
	RoundStartsAt int64  `json:"round_starts_at"`
	RoundEndsAt   int64  `json:"round_ends_at"`
	StakeLamports int64  `json:"stake_lamports"`
	Status        string `json:"status"`
}

func entryView(sa *model.Sale, q *model.QueueEntry) *queueEntryView {
	eff := effectivePosition(sa, q.Position)
	round := int64(0)
	if sa.WindowSize > 0 {
		round = int64(eff) / int64(sa.WindowSize)
	}
	start := sa.ClaimStart + round*sa.RoundDurationSecs
	return &queueEntryView{
		Pubkey:        q.Pubkey,
		Buyer:         q.Buyer,
		Position:      q.Position,
		EffectivePos:  eff,
		Round:         round,
		RoundStartsAt: start,
		RoundEndsAt:   start + sa.RoundDurationSecs,
		StakeLamports: q.StakeLamports,
		Status:        model.QueueEntryStatus(q.Status).String(),
	}
}

func effectivePosition(sa *model.Sale, position uint32) uint32 {
	if sa.TotalEntries == 0 {
		return 0
	}
	total := uint64(sa.TotalEntries)
	return uint32((sa.Randomness%total + uint64(position)%total) % total)
}

func phase(sa *model.Sale, now int64) string {
	switch {
	case now < sa.RegistrationStart:
		return "announced"
	case now < sa.RegistrationEnd:
		return "registration"
	case now < sa.RevealAt:
		return "reveal"
	case now < sa.ClaimStart:
		return "draw"
	default:
		return "claim"
	}
}

func (s *Server) handleWalletTickets(w http.ResponseWriter, r *http.Request) {
	wallet := r.PathValue("pubkey")
	tickets, err := s.store.ListTicketsForWallet(r.Context(), wallet)
	if err != nil {
		s.logger.Error("wallet tickets", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}

	type item struct {
		model.WalletTicket
		StatusText string `json:"status"`
	}
	items := make([]item, 0, len(tickets))
	for _, t := range tickets {
		items = append(items, item{WalletTicket: t, StatusText: model.TicketStatus(t.Status).String()})
	}
	writeJSON(w, http.StatusOK, map[string]any{"tickets": items})
}

func (s *Server) handleTicketByMint(w http.ResponseWriter, r *http.Request) {
	mint := r.PathValue("mint")
	ticket, err := s.store.GetTicketByMint(r.Context(), mint)
	if err != nil {
		s.logger.Error("ticket by mint", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	if ticket == nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "ticket not found"})
		return
	}
	type withStatus struct {
		model.WalletTicket
		StatusText string `json:"status"`
	}
	writeJSON(w, http.StatusOK, withStatus{WalletTicket: *ticket, StatusText: model.TicketStatus(ticket.Status).String()})
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}
