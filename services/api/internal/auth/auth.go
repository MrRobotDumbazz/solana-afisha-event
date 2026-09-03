package auth

import (
	"context"
	"crypto/ed25519"
	"crypto/hmac"
	crypto_rand "crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"sync"
	"time"

	"github.com/gagliardetto/solana-go"
)

const (
	nonceTTL     = 10 * time.Minute
	sessionTTL   = 24 * time.Hour
	nonceCleanup = time.Minute
)

type nonceRecord struct {
	wallet  solana.PublicKey
	message []byte
	expires time.Time
}

type Manager struct {
	mu     sync.Mutex
	nonces map[string]*nonceRecord
	secret []byte
	logger *slog.Logger
}

func NewManager(logger *slog.Logger) *Manager {
	secret := []byte(os.Getenv("AUTH_SECRET"))
	if len(secret) == 0 {
		secret = make([]byte, 32)
		if _, err := crypto_rand.Read(secret); err != nil {
			panic(err)
		}
	}
	m := &Manager{
		nonces: make(map[string]*nonceRecord),
		secret: secret,
		logger: logger,
	}
	go m.cleanupLoop()
	return m
}

func (m *Manager) cleanupLoop() {
	ticker := time.NewTicker(nonceCleanup)
	for range ticker.C {
		m.mu.Lock()
		now := time.Now()
		for id, rec := range m.nonces {
			if now.After(rec.expires) {
				delete(m.nonces, id)
			}
		}
		m.mu.Unlock()
	}
}

func (m *Manager) NewNonce(wallet solana.PublicKey) (string, []byte, error) {
	raw := make([]byte, 16)
	if _, err := crypto_rand.Read(raw); err != nil {
		return "", nil, err
	}
	id := hex.EncodeToString(raw)
	issued := time.Now()
	message := []byte(fmt.Sprintf(
		"Afisha.sol — вход\nКошелёк: %s\nNonce: %s\nВыдан: %s\nПодпись подтверждает владение этим кошельком.",
		wallet.String(), id, issued.UTC().Format(time.RFC3339),
	))

	m.mu.Lock()
	m.nonces[id] = &nonceRecord{
		wallet:  wallet,
		message: message,
		expires: issued.Add(nonceTTL),
	}
	m.mu.Unlock()
	return id, message, nil
}

func (m *Manager) Verify(wallet solana.PublicKey, nonceID string, signature []byte) error {
	m.mu.Lock()
	rec, ok := m.nonces[nonceID]
	if ok {
		delete(m.nonces, nonceID)
	}
	m.mu.Unlock()

	if !ok {
		return errors.New("nonce not found or already used")
	}
	if time.Now().After(rec.expires) {
		return errors.New("nonce expired")
	}
	if rec.wallet != wallet {
		return errors.New("nonce belongs to another wallet")
	}
	if len(signature) != ed25519.SignatureSize {
		return errors.New("bad signature length")
	}
	if !ed25519.Verify(ed25519.PublicKey(wallet.Bytes()), rec.message, signature) {
		return errors.New("signature verification failed")
	}
	return nil
}

type sessionClaims struct {
	Wallet string `json:"wallet"`
	Exp    int64  `json:"exp"`
}

func (m *Manager) IssueSession(wallet solana.PublicKey) (string, int64, error) {
	expiresAt := time.Now().Add(sessionTTL).Unix()
	payload, err := json.Marshal(sessionClaims{Wallet: wallet.String(), Exp: expiresAt})
	if err != nil {
		return "", 0, err
	}
	body := base64.RawURLEncoding.EncodeToString(payload)
	mac := hmac.New(sha256.New, m.secret)
	mac.Write([]byte(body))
	sig := base64.RawURLEncoding.EncodeToString(mac.Sum(nil))
	return fmt.Sprintf("%s.%s", body, sig), expiresAt, nil
}

func (m *Manager) ParseSession(token string) (solana.PublicKey, error) {
	var wallet solana.PublicKey
	dot := -1
	for i := 0; i < len(token); i++ {
		if token[i] == '.' {
			dot = i
			break
		}
	}
	if dot <= 0 || dot == len(token)-1 {
		return wallet, errors.New("malformed token")
	}
	body, sig := token[:dot], token[dot+1:]
	mac := hmac.New(sha256.New, m.secret)
	mac.Write([]byte(body))
	expected := base64.RawURLEncoding.EncodeToString(mac.Sum(nil))
	if !hmac.Equal([]byte(expected), []byte(sig)) {
		return wallet, errors.New("bad token signature")
	}
	payload, err := base64.RawURLEncoding.DecodeString(body)
	if err != nil {
		return wallet, errors.New("bad token payload")
	}
	var claims sessionClaims
	if err := json.Unmarshal(payload, &claims); err != nil {
		return wallet, errors.New("bad token claims")
	}
	if time.Now().Unix() > claims.Exp {
		return wallet, errors.New("token expired")
	}
	return solana.PublicKeyFromBase58(claims.Wallet)
}

type walletCtxKey struct{}

type Server struct {
	manager *Manager
	logger  *slog.Logger
}

func NewServer(manager *Manager, logger *slog.Logger) *Server {
	return &Server{manager: manager, logger: logger}
}

func (s *Server) handleNonce(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Wallet string `json:"wallet"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Wallet == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "wallet required"})
		return
	}
	wallet, err := solana.PublicKeyFromBase58(req.Wallet)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid wallet"})
		return
	}
	nonceID, message, err := s.manager.NewNonce(wallet)
	if err != nil {
		s.logger.Error("nonce", "err", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{
		"nonce":   nonceID,
		"message": string(message),
	})
}

func (s *Server) handleVerify(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Wallet    string `json:"wallet"`
		Nonce     string `json:"nonce"`
		Signature string `json:"signature"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "bad body"})
		return
	}
	wallet, err := solana.PublicKeyFromBase58(req.Wallet)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid wallet"})
		return
	}
	signature, err := base64.StdEncoding.DecodeString(req.Signature)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "signature must be base64"})
		return
	}
	if err := s.manager.Verify(wallet, req.Nonce, signature); err != nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": err.Error()})
		return
	}
	token, expiresAt, err := s.manager.IssueSession(wallet)
	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal"})
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{
		"token":      token,
		"wallet":     wallet.String(),
		"expires_at": expiresAt,
	})
}

func (s *Server) handleMe(w http.ResponseWriter, r *http.Request) {
	wallet := r.Context().Value(walletCtxKey{}).(solana.PublicKey)
	writeJSON(w, http.StatusOK, map[string]string{"wallet": wallet.String()})
}

type Route struct {
	Pattern string
	Handler http.HandlerFunc
}

func (s *Server) Routes() []Route {
	return []Route{
		{"POST /api/v1/auth/nonce", s.handleNonce},
		{"POST /api/v1/auth/verify", s.handleVerify},
		{"GET /api/v1/auth/me", s.requireAuth(s.handleMe)},
	}
}

func (s *Server) requireAuth(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		header := r.Header.Get("Authorization")
		if len(header) <= 8 || header[:7] != "Bearer " {
			writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "missing bearer token"})
			return
		}
		wallet, err := s.manager.ParseSession(header[7:])
		if err != nil {
			writeJSON(w, http.StatusUnauthorized, map[string]string{"error": err.Error()})
			return
		}
		ctx := context.WithValue(r.Context(), walletCtxKey{}, wallet)
		next(w, r.WithContext(ctx))
	}
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}
