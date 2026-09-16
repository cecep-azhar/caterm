package host

import (
	"context"
	"path/filepath"
	"testing"

	"caterm/internal/store"
)

func TestGroupService(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("Migrate failed: %v", err)
	}

	svc := NewGroupService(db)

	// Create
	g1, err := svc.Create(ctx, "QA_TEST_grp1", 10)
	if err != nil {
		t.Fatalf("Create failed: %v", err)
	}
	if g1.ID == "" || g1.CreatedAt == "" || g1.UpdatedAt == "" || g1.CreatedAt != g1.UpdatedAt {
		t.Errorf("Invalid created group: %+v", g1)
	}

	// Update
	oldUpdated := g1.UpdatedAt
	g2, err := svc.Update(ctx, g1.ID, "QA_TEST_grp1_renamed", 20)
	if err != nil {
		t.Fatalf("Update failed: %v", err)
	}
	if g2.Name != "QA_TEST_grp1_renamed" || g2.SortOrder != 20 {
		t.Errorf("Update did not change fields: %+v", g2)
	}
	if g2.CreatedAt != g1.CreatedAt {
		t.Errorf("CreatedAt changed during update")
	}
	if g2.UpdatedAt == oldUpdated {
		t.Errorf("UpdatedAt did not change")
	}

	// List
	list, err := svc.List(ctx)
	if err != nil {
		t.Fatalf("List failed: %v", err)
	}
	if len(list) != 1 {
		t.Fatalf("Expected 1 group, got %d", len(list))
	}

	// Soft Delete
	if err := svc.Delete(ctx, g1.ID); err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	listAfterDelete, _ := svc.List(ctx)
	if len(listAfterDelete) != 0 {
		t.Errorf("Expected 0 groups after delete, got %d", len(listAfterDelete))
	}

	// Direct fetch should show deleted_at
	deletedGroup, err := svc.GetByID(ctx, g1.ID)
	if err != nil {
		t.Fatalf("GetByID failed: %v", err)
	}
	if deletedGroup.DeletedAt == nil {
		t.Errorf("DeletedAt is nil after soft delete")
	}
}
