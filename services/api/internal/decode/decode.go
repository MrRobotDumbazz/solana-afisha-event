package decode

import (
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"fmt"

	"afisha/api/internal/model"
)

type reader struct {
	data []byte
	off  int
}

func (r *reader) take(n int) ([]byte, error) {
	if r.off+n > len(r.data) {
		return nil, fmt.Errorf("borsh: unexpected end of data at %d (need %d)", r.off, n)
	}
	b := r.data[r.off : r.off+n]
	r.off += n
	return b, nil
}

func (r *reader) u8() (uint8, error) {
	b, err := r.take(1)
	if err != nil {
		return 0, err
	}
	return b[0], nil
}

func (r *reader) bool() (bool, error) {
	b, err := r.take(1)
	if err != nil {
		return false, err
	}
	return b[0] != 0, nil
}

func (r *reader) u32() (uint32, error) {
	b, err := r.take(4)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint32(b), nil
}

func (r *reader) u64() (uint64, error) {
	b, err := r.take(8)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint64(b), nil
}

func (r *reader) i64() (int64, error) {
	v, err := r.u64()
	return int64(v), err
}

func (r *reader) pubkey() (string, error) {
	b, err := r.take(32)
	if err != nil {
		return "", err
	}
	return encodeBase58(b), nil
}

func (r *reader) str() (string, error) {
	n, err := r.u32()
	if err != nil {
		return "", err
	}
	if uint64(n) > uint64(len(r.data)-r.off) {
		return "", errors.New("borsh: string length out of range")
	}
	b, err := r.take(int(n))
	if err != nil {
		return "", err
	}
	return string(b), nil
}

func discriminator(name string) []byte {
	h := sha256.Sum256([]byte("account:" + name))
	return h[:8]
}

var (
	DiscEvent      = discriminator("Event")
	DiscTicket     = discriminator("Ticket")
	DiscSaleState  = discriminator("SaleState")
	DiscQueueEntry = discriminator("QueueEntry")
)

type Kind int

const (
	KindUnknown Kind = iota
	KindEvent
	KindTicket
	KindSaleState
	KindQueueEntry
)

func Classify(data []byte) Kind {
	if len(data) < 8 {
		return KindUnknown
	}
	switch {
	case matches(data, DiscEvent):
		return KindEvent
	case matches(data, DiscTicket):
		return KindTicket
	case matches(data, DiscSaleState):
		return KindSaleState
	case matches(data, DiscQueueEntry):
		return KindQueueEntry
	}
	return KindUnknown
}

func matches(data, disc []byte) bool {
	for i := 0; i < 8; i++ {
		if data[i] != disc[i] {
			return false
		}
	}
	return true
}

func Event(data []byte) (*model.Event, error) {
	r := &reader{data: data}
	if _, err := r.take(8); err != nil {
		return nil, err
	}
	e := &model.Event{}
	var err error
	if e.Organizer, err = r.pubkey(); err != nil {
		return nil, err
	}
	if e.Slug, err = r.str(); err != nil {
		return nil, err
	}
	if e.Title, err = r.str(); err != nil {
		return nil, err
	}
	if e.Description, err = r.str(); err != nil {
		return nil, err
	}
	if e.Venue, err = r.str(); err != nil {
		return nil, err
	}
	if e.City, err = r.str(); err != nil {
		return nil, err
	}
	if e.ImageURI, err = r.str(); err != nil {
		return nil, err
	}
	if e.StartsAt, err = r.i64(); err != nil {
		return nil, err
	}
	if e.EndsAt, err = r.i64(); err != nil {
		return nil, err
	}
	if e.TicketPriceLamport, err = r.i64(); err != nil {
		return nil, err
	}
	if e.Capacity, err = r.u32(); err != nil {
		return nil, err
	}
	if e.TicketsSold, err = r.u32(); err != nil {
		return nil, err
	}
	if e.HotSale, err = r.bool(); err != nil {
		return nil, err
	}
	if e.Status, err = r.u8(); err != nil {
		return nil, err
	}
	return e, nil
}

func Ticket(data []byte) (*model.Ticket, error) {
	r := &reader{data: data}
	if _, err := r.take(8); err != nil {
		return nil, err
	}
	t := &model.Ticket{}
	var err error
	if t.EventPubkey, err = r.pubkey(); err != nil {
		return nil, err
	}
	if t.Buyer, err = r.pubkey(); err != nil {
		return nil, err
	}
	if t.Mint, err = r.pubkey(); err != nil {
		return nil, err
	}
	if t.Status, err = r.u8(); err != nil {
		return nil, err
	}
	if t.CheckedInAt, err = r.i64(); err != nil {
		return nil, err
	}
	return t, nil
}

func SaleState(data []byte) (*model.Sale, error) {
	r := &reader{data: data}
	if _, err := r.take(8); err != nil {
		return nil, err
	}
	s := &model.Sale{}
	var err error
	if s.EventPubkey, err = r.pubkey(); err != nil {
		return nil, err
	}
	if s.RegistrationStart, err = r.i64(); err != nil {
		return nil, err
	}
	if s.RegistrationEnd, err = r.i64(); err != nil {
		return nil, err
	}
	if s.RevealAt, err = r.i64(); err != nil {
		return nil, err
	}
	if s.ClaimStart, err = r.i64(); err != nil {
		return nil, err
	}
	if s.RoundDurationSecs, err = r.i64(); err != nil {
		return nil, err
	}
	if s.StakeLamports, err = r.i64(); err != nil {
		return nil, err
	}
	if s.WindowSize, err = r.u32(); err != nil {
		return nil, err
	}
	if s.TotalEntries, err = r.u32(); err != nil {
		return nil, err
	}
	if s.Randomness, err = r.u64(); err != nil {
		return nil, err
	}
	if s.Settled, err = r.bool(); err != nil {
		return nil, err
	}
	if s.Claimed, err = r.u32(); err != nil {
		return nil, err
	}
	if s.SettledCount, err = r.u32(); err != nil {
		return nil, err
	}
	if s.ForfeitedCount, err = r.u32(); err != nil {
		return nil, err
	}
	if _, err = r.u8(); err != nil {
		return nil, err
	}
	return s, nil
}

func QueueEntry(data []byte) (*model.QueueEntry, error) {
	r := &reader{data: data}
	if _, err := r.take(8); err != nil {
		return nil, err
	}
	q := &model.QueueEntry{}
	var err error
	if q.EventPubkey, err = r.pubkey(); err != nil {
		return nil, err
	}
	if q.Buyer, err = r.pubkey(); err != nil {
		return nil, err
	}
	if q.Position, err = r.u32(); err != nil {
		return nil, err
	}
	if q.StakeLamports, err = r.i64(); err != nil {
		return nil, err
	}
	if q.Status, err = r.u8(); err != nil {
		return nil, err
	}
	return q, nil
}
