package decode

import (
	"encoding/binary"
	"testing"
)

func appendStr(b []byte, s string) []byte {
	var l [4]byte
	binary.LittleEndian.PutUint32(l[:], uint32(len(s)))
	b = append(b, l[:]...)
	return append(b, s...)
}

func appendU32(b []byte, v uint32) []byte {
	var x [4]byte
	binary.LittleEndian.PutUint32(x[:], v)
	return append(b, x[:]...)
}

func appendU64(b []byte, v uint64) []byte {
	var x [8]byte
	binary.LittleEndian.PutUint64(x[:], v)
	return append(b, x[:]...)
}

func appendI64(b []byte, v int64) []byte {
	return appendU64(b, uint64(v))
}

func TestClassifyKnownDiscriminators(t *testing.T) {
	golden := map[string][8]byte{
		"Event":      {125, 192, 125, 158, 9, 115, 152, 233},
		"Ticket":     {41, 228, 24, 165, 78, 90, 235, 200},
		"SaleState":  {21, 13, 249, 194, 208, 96, 243, 65},
		"QueueEntry": {211, 46, 29, 56, 240, 146, 48, 178},
	}
	if got := [8]byte(DiscEvent); got != golden["Event"] {
		t.Fatalf("event disc mismatch: %v", got)
	}
	if got := [8]byte(DiscTicket); got != golden["Ticket"] {
		t.Fatalf("ticket disc mismatch: %v", got)
	}
	if got := [8]byte(DiscSaleState); got != golden["SaleState"] {
		t.Fatalf("sale disc mismatch: %v", got)
	}
	if got := [8]byte(DiscQueueEntry); got != golden["QueueEntry"] {
		t.Fatalf("queue disc mismatch: %v", got)
	}
}

func TestDecodeTicket(t *testing.T) {
	organizer := make([]byte, 32)
	organizer[0] = 1
	event := make([]byte, 32)
	event[0] = 2
	buyer := make([]byte, 32)
	buyer[0] = 3
	mint := make([]byte, 32)
	mint[0] = 4

	var data []byte
	data = append(data, DiscTicket...)
	data = append(data, event...)
	data = append(data, buyer...)
	data = append(data, mint...)
	data = append(data, byte(0))       // status Valid
	data = appendU64(data, 1700000000) // checked_in_at

	tk, err := Ticket(data)
	if err != nil {
		t.Fatal(err)
	}
	if tk.CheckedInAt != 1700000000 || tk.Status != 0 {
		t.Fatalf("bad decode: %+v", tk)
	}
	if len(tk.Buyer) < 43 || len(tk.Buyer) > 44 {
		t.Fatalf("buyer should be base58 pubkey, got %q", tk.Buyer)
	}
}

func TestDecodeEvent(t *testing.T) {
	var data []byte
	data = append(data, DiscEvent...)
	data = append(data, make([]byte, 32)...) // organizer
	data = appendStr(data, "solana-break-2026")
	data = appendStr(data, "Break 2026")
	data = appendStr(data, "desc")
	data = appendStr(data, "Loft")
	data = appendStr(data, "Almaty")
	data = appendStr(data, "https://img")
	data = appendI64(data, 1000)
	data = appendI64(data, 2000)
	data = appendI64(data, 50000000)
	data = appendU32(data, 100)
	data = appendU32(data, 3)
	data = append(data, byte(1)) // hot_sale
	data = append(data, byte(0)) // active

	ev, err := Event(data)
	if err != nil {
		t.Fatal(err)
	}
	if ev.Slug != "solana-break-2026" || ev.Title != "Break 2026" || ev.City != "Almaty" {
		t.Fatalf("bad strings: %+v", ev)
	}
	if ev.Capacity != 100 || ev.TicketsSold != 3 || !ev.HotSale || ev.Status != 0 {
		t.Fatalf("bad scalars: %+v", ev)
	}
	if ev.StartsAt != 1000 || ev.EndsAt != 2000 || ev.TicketPriceLamport != 50000000 {
		t.Fatalf("bad times: %+v", ev)
	}
}

func TestDecodeSaleAndQueue(t *testing.T) {
	var s []byte
	s = append(s, DiscSaleState...)
	s = append(s, make([]byte, 32)...)
	s = appendI64(s, 60)
	s = appendI64(s, 600)
	s = appendI64(s, 700)
	s = appendI64(s, 800)
	s = appendI64(s, 300)
	s = appendI64(s, 50000000)
	s = appendU32(s, 10)
	s = appendU32(s, 42)
	s = appendU64(s, 123456789)
	s = append(s, byte(1))
	s = appendU32(s, 5)
	s = appendU32(s, 2)
	s = appendU32(s, 1)
	s = append(s, byte(255))

	sale, err := SaleState(s)
	if err != nil {
		t.Fatal(err)
	}
	if sale.TotalEntries != 42 || sale.WindowSize != 10 || !sale.Settled ||
		sale.Claimed != 5 || sale.SettledCount != 2 || sale.ForfeitedCount != 1 ||
		sale.Randomness != 123456789 {
		t.Fatalf("bad sale: %+v", sale)
	}

	var q []byte
	q = append(q, DiscQueueEntry...)
	q = append(q, make([]byte, 32)...)
	q = append(q, make([]byte, 32)...)
	q = appendU32(q, 7)
	q = appendI64(q, 50000000)
	q = append(q, byte(0))

	entry, err := QueueEntry(q)
	if err != nil {
		t.Fatal(err)
	}
	if entry.Position != 7 || entry.StakeLamports != 50000000 || entry.Status != 0 {
		t.Fatalf("bad entry: %+v", entry)
	}
}

func TestTruncatedDataFails(t *testing.T) {
	if _, err := Event(DiscEvent); err == nil {
		t.Fatal("expected error on truncated event")
	}
}
