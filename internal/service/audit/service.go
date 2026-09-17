package audit

import (
	"context"
	"time"

	"github.com/google/uuid"

	"caterm/internal/store"
)

type AuditLog struct {
	ID        string    `json:"id"`
	HostID    string    `json:"host_id"`
	Command   string    `json:"command"`
	Output    string    `json:"output"`
	ExitCode  int       `json:"exit_code"`
	CreatedAt time.Time `json:"created_at"`
}

type Service struct {
	db *store.DB
}

func NewService(db *store.DB) *Service {
	return &Service{db: db}
}

func (s *Service) LogCommand(ctx context.Context, hostID, command, output string, exitCode int) error {
	now := time.Now().UTC()
	id := uuid.NewString()

	// Truncate output to 5KB max to prevent bloat/leakage
	if len(output) > 5000 {
		output = output[:5000] + "...[TRUNCATED]"
	}

	_, err := s.db.ExecContext(ctx, `
		INSERT INTO audit_logs (id, host_id, command, output, exit_code, created_at)
		VALUES (?, ?, ?, ?, ?, ?)
	`, id, hostID, command, output, exitCode, now.Format(time.RFC3339Nano))
	return err
}

func (s *Service) ListLogs(ctx context.Context, hostID string, limit, offset int) ([]AuditLog, error) {
	q := `
		SELECT id, host_id, command, output, exit_code, created_at 
		FROM audit_logs
	`
	args := []any{}

	if hostID != "" {
		q += ` WHERE host_id = ?`
		args = append(args, hostID)
	}

	q += ` ORDER BY created_at DESC`

	if limit > 0 {
		q += ` LIMIT ?`
		args = append(args, limit)
	}
	if offset > 0 {
		q += ` OFFSET ?`
		args = append(args, offset)
	}

	rows, err := s.db.QueryContext(ctx, q, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var logs []AuditLog
	for rows.Next() {
		var l AuditLog
		var createdStr string
		if err := rows.Scan(&l.ID, &l.HostID, &l.Command, &l.Output, &l.ExitCode, &createdStr); err != nil {
			return nil, err
		}
		if t, err := time.Parse(time.RFC3339, createdStr); err == nil {
			l.CreatedAt = t
		}
		logs = append(logs, l)
	}
	return logs, rows.Err()
}

func (s *Service) ClearLogs(ctx context.Context, hostID string) error {
	if hostID != "" {
		_, err := s.db.ExecContext(ctx, `DELETE FROM audit_logs WHERE host_id = ?`, hostID)
		return err
	}
	_, err := s.db.ExecContext(ctx, `DELETE FROM audit_logs`)
	return err
}
