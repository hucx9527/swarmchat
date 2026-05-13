// Package prekey implements prekey bundle storage per SCP §4.6.
//
// The relay node stores prekey bundles for users, allowing other users
// to fetch them when initiating encrypted sessions (X3DH).
// One-time prekeys (OPK) are marked as used after being served once.
package prekey

import (
	"context"
	"database/sql"
	"fmt"
	"time"
)

// Store manages prekey bundles in the database
type Store struct {
	db *sql.DB
}

// Bundle represents a user's prekey bundle
type Bundle struct {
	DID             string
	IdentityKey     string
	SignedPrekey    string
	SPKSignature    string
	OneTimePrekeys  []string
	UploadedAt      time.Time
}

// NewStore creates a new prekey store
func NewStore(db *sql.DB) *Store {
	return &Store{db: db}
}

// Upload stores a prekey bundle for a DID.
// Replaces any existing bundle (SPK update) and appends new OPKs.
func (s *Store) Upload(ctx context.Context, bundle *Bundle) error {
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin tx: %w", err)
	}
	defer tx.Rollback()

	// Upsert the identity and signed prekey
	_, err = tx.ExecContext(ctx, `
		INSERT INTO prekey_bundles (did, identity_key, signed_prekey, spk_signature, uploaded_at)
		VALUES (?, ?, ?, ?, ?)
		ON CONFLICT(did) DO UPDATE SET
			identity_key = excluded.identity_key,
			signed_prekey = excluded.signed_prekey,
			spk_signature = excluded.spk_signature,
			uploaded_at = excluded.uploaded_at
	`, bundle.DID, bundle.IdentityKey, bundle.SignedPrekey, bundle.SPKSignature, bundle.UploadedAt)
	if err != nil {
		return fmt.Errorf("upsert bundle: %w", err)
	}

	// Insert new one-time prekeys
	for _, opk := range bundle.OneTimePrekeys {
		_, err = tx.ExecContext(ctx, `
			INSERT INTO one_time_prekeys (did, prekey, used)
			VALUES (?, ?, 0)
		`, bundle.DID, opk)
		if err != nil {
			return fmt.Errorf("insert OPK: %w", err)
		}
	}

	return tx.Commit()
}

// Get retrieves the prekey bundle for a DID.
// Returns the identity key, signed prekey, and available one-time prekeys.
func (s *Store) Get(ctx context.Context, did string) (*Bundle, error) {
	var bundle Bundle
	bundle.DID = did

	err := s.db.QueryRowContext(ctx, `
		SELECT identity_key, signed_prekey, spk_signature, uploaded_at
		FROM prekey_bundles
		WHERE did = ?
	`, did).Scan(&bundle.IdentityKey, &bundle.SignedPrekey, &bundle.SPKSignature, &bundle.UploadedAt)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("no prekey bundle for DID: %s", did)
		}
		return nil, fmt.Errorf("query bundle: %w", err)
	}

	// Get unused OPKs
	rows, err := s.db.QueryContext(ctx, `
		SELECT prekey FROM one_time_prekeys
		WHERE did = ? AND used = 0
		LIMIT 100
	`, did)
	if err != nil {
		return nil, fmt.Errorf("query OPKs: %w", err)
	}
	defer rows.Close()

	for rows.Next() {
		var opk string
		if err := rows.Scan(&opk); err != nil {
			return nil, fmt.Errorf("scan OPK: %w", err)
		}
		bundle.OneTimePrekeys = append(bundle.OneTimePrekeys, opk)
	}

	return &bundle, nil
}

// UseOneTimePrekey marks a specific OPK as used
func (s *Store) UseOneTimePrekey(ctx context.Context, did, opk string) error {
	_, err := s.db.ExecContext(ctx, `
		UPDATE one_time_prekeys SET used = 1
		WHERE did = ? AND prekey = ?
	`, did, opk)
	return err
}

// GetOPKCount returns the number of unused OPKs for a DID
func (s *Store) GetOPKCount(ctx context.Context, did string) (int, error) {
	var count int
	err := s.db.QueryRowContext(ctx, `
		SELECT COUNT(*) FROM one_time_prekeys
		WHERE did = ? AND used = 0
	`, did).Scan(&count)
	return count, err
}

// CleanupExpired removes OPKs older than the given duration
func (s *Store) CleanupExpired(ctx context.Context, maxAge time.Duration) error {
	cutoff := time.Now().Add(-maxAge)
	_, err := s.db.ExecContext(ctx, `
		DELETE FROM one_time_prekeys
		WHERE used = 1 AND created_at < ?
	`, cutoff)
	return err
}
