package model

type EventStatus uint8

const (
	EventStatusActive EventStatus = iota
	EventStatusCancelled
)

func (s EventStatus) String() string {
	switch s {
	case EventStatusActive:
		return "active"
	case EventStatusCancelled:
		return "cancelled"
	default:
		return "unknown"
	}
}

type TicketStatus uint8

const (
	TicketStatusValid TicketStatus = iota
	TicketStatusUsed
)

func (s TicketStatus) String() string {
	switch s {
	case TicketStatusValid:
		return "valid"
	case TicketStatusUsed:
		return "used"
	default:
		return "unknown"
	}
}

type QueueEntryStatus uint8

const (
	QueueStaked QueueEntryStatus = iota
	QueueClaimed
	QueueSettled
	QueueForfeited
)

func (s QueueEntryStatus) String() string {
	switch s {
	case QueueStaked:
		return "staked"
	case QueueClaimed:
		return "claimed"
	case QueueSettled:
		return "settled"
	case QueueForfeited:
		return "forfeited"
	default:
		return "unknown"
	}
}

type Event struct {
	Pubkey             string `json:"pubkey"`
	Organizer          string `json:"organizer"`
	Slug               string `json:"slug"`
	Title              string `json:"title"`
	Description        string `json:"description"`
	Venue              string `json:"venue"`
	City               string `json:"city"`
	ImageURI           string `json:"image_uri"`
	StartsAt           int64  `json:"starts_at"`
	EndsAt             int64  `json:"ends_at"`
	TicketPriceLamport int64  `json:"ticket_price_lamports"`
	Capacity           uint32 `json:"capacity"`
	TicketsSold        uint32 `json:"tickets_sold"`
	HotSale            bool   `json:"hot_sale"`
	SaleConfigured     bool   `json:"sale_configured"`
	Status             uint8  `json:"-"`
}

type Sale struct {
	EventPubkey       string `json:"event_pubkey"`
	RegistrationStart int64  `json:"registration_start"`
	RegistrationEnd   int64  `json:"registration_end"`
	RevealAt          int64  `json:"reveal_at"`
	ClaimStart        int64  `json:"claim_start"`
	RoundDurationSecs int64  `json:"round_duration_secs"`
	StakeLamports     int64  `json:"stake_lamports"`
	WindowSize        uint32 `json:"window_size"`
	TotalEntries      uint32 `json:"total_entries"`
	Randomness        uint64 `json:"-"`
	Settled           bool   `json:"settled"`
	Claimed           uint32 `json:"claimed"`
	SettledCount      uint32 `json:"settled_count"`
	ForfeitedCount    uint32 `json:"forfeited_count"`
}

type Ticket struct {
	Pubkey      string `json:"pubkey"`
	EventPubkey string `json:"event_pubkey"`
	Buyer       string `json:"buyer"`
	Mint        string `json:"mint"`
	Status      uint8  `json:"-"`
	CheckedInAt int64  `json:"checked_in_at"`
}

type QueueEntry struct {
	Pubkey        string `json:"pubkey"`
	EventPubkey   string `json:"event_pubkey"`
	Buyer         string `json:"buyer"`
	Position      uint32 `json:"position"`
	StakeLamports int64  `json:"stake_lamports"`
	Status        uint8  `json:"-"`
}

type QueueView struct {
	Sale
	Phase            string `json:"phase"`
	Pending          uint32 `json:"pending"`
	CurrentRound     int64  `json:"current_round"`
	RoundServingFrom uint32 `json:"round_serving_from"`
	RoundServingTo   uint32 `json:"round_serving_to"`
}

type WalletTicket struct {
	Ticket
	EventTitle  string `json:"event_title"`
	EventStarts int64  `json:"event_starts_at"`
	EventEnds   int64  `json:"event_ends_at"`
	EventCity   string `json:"event_city"`
	EventStatus string `json:"event_status"`
}

type EventDetails struct {
	Event
	StatusText  string `json:"status"`
	TicketsLeft uint32 `json:"tickets_left"`
	Sale        *Sale  `json:"sale,omitempty"`
}
