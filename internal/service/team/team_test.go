package team_test

import (
	"context"
	"testing"

	"caterm/internal/service/team"
	"caterm/internal/store"
)

func TestTeamServiceCRUD(t *testing.T) {
	db, err := store.Open(":memory:")
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("failed to migrate db: %v", err)
	}

	svc := team.NewTeamService(db)

	// Create
	tm, err := svc.Create(ctx, team.TeamInput{
		Name: "Test Team",
	})
	if err != nil {
		t.Fatalf("Create failed: %v", err)
	}
	if tm.ID == "" || tm.Name != "Test Team" {
		t.Fatalf("unexpected team created: %+v", tm)
	}

	// List
	list, err := svc.List(ctx)
	if err != nil {
		t.Fatalf("List failed: %v", err)
	}
	if len(list) != 1 {
		t.Fatalf("expected 1 team, got %d", len(list))
	}

	// Update
	updated, err := svc.Update(ctx, tm.ID, team.TeamInput{
		Name: "Updated Team",
	})
	if err != nil {
		t.Fatalf("Update failed: %v", err)
	}
	if updated.Name != "Updated Team" {
		t.Fatalf("unexpected updated team: %+v", updated)
	}

	// Delete
	if err := svc.Delete(ctx, tm.ID); err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	listAfter, err := svc.List(ctx)
	if err != nil {
		t.Fatalf("List after delete failed: %v", err)
	}
	if len(listAfter) != 0 {
		t.Fatalf("expected 0 teams after delete, got %d", len(listAfter))
	}
}
