// Package storage provides SQLite-based persistence for the relay node.
// Implements offline message storage per SCP §5.5.
package storage

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// SQLiteStore wraps the SQLite database connection
type SQLiteStore struct {
	db *sql.DB
}

// OfflineStore handles offline message persistence
type OfflineStore struct {
	db     *sql.DB
	maxAge time.Duration
}

// OfflineMessage represents a stored offline message
type OfflineMessage struct {
	ID        string
	FromDID   string
	ToDID     string
	Envelope  string
	Timestamp int64
	TTL       uint32
	Delivered bool
}

// NewSQLiteStore opens and configures a SQLite database
func NewSQLiteStore(path string) (*SQLiteStore, error) {
	db, err := sql.Open("sqlite3", path+"?_journal_mode=WAL&_busy_timeout=5000")
	if err != nil {
		return nil, fmt.Errorf("open database: %w", err)
	}

	// Connection pool configuration
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(5 * time.Minute)

	// Enable WAL mode and foreign keys
	pragmas := []string{
		"PRAGMA journal_mode=WAL",
		"PRAGMA foreign_keys=ON",
		"PRAGMA busy_timeout=5000",
	}
	for _, pragma := range pragmas {
		if _, err := db.Exec(pragma); err != nil {
			return nil, fmt.Errorf("set pragma: %w", err)
		}
	}

	return &SQLiteStore{db: db}, nil
}

// DB returns the underlying database connection
func (s *SQLiteStore) DB() *sql.DB {
	return s.db
}

// Close closes the database connection
func (s *SQLiteStore) Close() error {
	return s.db.Close()
}

// Migrate creates the database schema
func (s *SQLiteStore) Migrate() error {
	migrations := []string{
		// Prekey bundles table (P2-2)
		`CREATE TABLE IF NOT EXISTS prekey_bundles (
			did TEXT PRIMARY KEY,
			identity_key TEXT NOT NULL,
			signed_prekey TEXT NOT NULL,
			spk_signature TEXT NOT NULL,
			uploaded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
		)`,

		// One-time prekeys table (P2-2)
		`CREATE TABLE IF NOT EXISTS one_time_prekeys (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			did TEXT NOT NULL,
			prekey TEXT NOT NULL,
			used INTEGER NOT NULL DEFAULT 0,
			created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
		)`,

		`CREATE INDEX IF NOT EXISTS idx_opk_did_used ON one_time_prekeys(did, used)`,

		// Offline messages table (P2-3)
		`CREATE TABLE IF NOT EXISTS offline_messages (
			id TEXT PRIMARY KEY,
			from_did TEXT NOT NULL,
			to_did TEXT NOT NULL,
			envelope TEXT NOT NULL,
			timestamp INTEGER NOT NULL,
			ttl INTEGER NOT NULL DEFAULT 604800,
			delivered INTEGER NOT NULL DEFAULT 0,
			delivered_at TIMESTAMP,
			stored_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
		)`,

		`CREATE INDEX IF NOT EXISTS idx_offline_to_did ON offline_messages(to_did, delivered, timestamp)`,
		`CREATE INDEX IF NOT EXISTS idx_offline_expires ON offline_messages(stored_at, ttl)`,

		// Message acknowledgements table
		`CREATE TABLE IF NOT EXISTS message_acks (
			message_id TEXT PRIMARY KEY,
			status TEXT NOT NULL DEFAULT 'delivered',
			ack_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
		)`,
	}

	for _, migration := range migrations {
		if _, err := s.db.Exec(migration); err != nil {
			return fmt.Errorf("migration failed: %w\nSQL: %s", err, migration)
		}
	}

	return nil
}

// NewOfflineStore creates a new offline message store
func NewOfflineStore(store *SQLiteStore, maxAge time.Duration) *OfflineStore {
	return &OfflineStore{
		db:     store.DB(),
		maxAge: maxAge,
	}
}

// Store saves an offline message
func (s *OfflineStore) Store(ctx context.Context, msg *OfflineMessage) error {
	if msg.ID == "" {
		return fmt.Errorf("message ID is required")
	}

	_, err := s.db.ExecContext(ctx, `
		INSERT INTO offline_messages (id, from_did, to_did, envelope, timestamp, ttl, delivered)
		VALUES (?, ?, ?, ?, ?, ?, 0)
		ON CONFLICT(id) DO NOTHING
	`, msg.ID, msg.FromDID, msg.ToDID, msg.Envelope, msg.Timestamp, msg.TTL)
	if err != nil {
		return fmt.Errorf("store message: %w", err)
	}

	return nil
}

// Sync retrieves undelivered messages for a DID since a given timestamp.
// Messages are returned in chronological order, limited by the limit parameter.
func (s *OfflineStore) Sync(ctx context.Context, did string, since int64, limit int) ([]*OfflineMessage, error) {
	cutoff := time.Now().Unix() - int64(s.maxAge.Seconds())

	rows, err := s.db.QueryContext(ctx, `
		SELECT id, from_did, to_did, envelope, timestamp, ttl
		FROM offline_messages
		WHERE to_did = ?
		AND delivered = 0
		AND timestamp > ?
		AND (ttl = 0 OR (stored_at + ttl) > ?)
		AND timestamp >= ?
		ORDER BY timestamp ASC
		LIMIT ?
	`, did, since, cutoff, since, limit)
	if err != nil {
		return nil, fmt.Errorf("query messages: %w", err)
	}
	defer rows.Close()

	var messages []*OfflineMessage
	for rows.Next() {
		msg := &OfflineMessage{ToDID: did, Delivered: false}
		if err := rows.Scan(&msg.ID, &msg.FromDID, &msg.ToDID, &msg.Envelope, &msg.Timestamp, &msg.TTL); err != nil {
			return nil, fmt.Errorf("scan message: %w", err)
		}
		messages = append(messages, msg)
	}

	return messages, nil
}

// MarkDelivered marks a message as delivered
func (s *OfflineStore) MarkDelivered(ctx context.Context, id string) error {
	_, err := s.db.ExecContext(ctx, `
		UPDATE offline_messages
		SET delivered = 1, delivered_at = ?
		WHERE id = ?
	`, time.Now(), id)
	return err
}

// CleanupExpired removes expired and delivered messages
func (s *OfflineStore) CleanupExpired(ctx context.Context) error {
	// Remove messages that have exceeded their TTL
	_, err := s.db.ExecContext(ctx, `
		DELETE FROM offline_messages
		WHERE (ttl > 0 AND (stored_at + ttl) < ?)
		OR (delivered = 1 AND delivered_at < ?)
	`, time.Now().Unix(), time.Now().Add(-1*time.Hour).Unix())
	return err
}

// GetStorageStats returns storage statistics
func (s *OfflineStore) GetStorageStats(ctx context.Context) (map[string]interface{}, error) {
	stats := make(map[string]interface{})

	var totalMsg int
	if err := s.db.QueryRowContext(ctx, `SELECT COUNT(*) FROM offline_messages`).Scan(&totalMsg); err != nil {
		return nil, err
	}
	stats["total_messages"] = totalMsg

	var undelivered int
	if err := s.db.QueryRowContext(ctx, `SELECT COUNT(*) FROM offline_messages WHERE delivered = 0`).Scan(&undelivered); err != nil {
		return nil, err
	}
	stats["undelivered_messages"] = undelivered

	var uniqueUsers int
	if err := s.db.QueryRowContext(ctx, `SELECT COUNT(DISTINCT to_did) FROM offline_messages`).Scan(&uniqueUsers); err != nil {
		return nil, err
	}
	stats["unique_users"] = uniqueUsers

	return stats, nil
}
