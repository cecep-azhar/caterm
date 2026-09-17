package snippet_test

import (
	"context"
	"testing"

	"caterm/internal/service/snippet"
	"caterm/internal/store"
)

func TestSnippetServiceCRUD(t *testing.T) {
	db, err := store.Open(":memory:")
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("failed to migrate db: %v", err)
	}

	svc := snippet.NewSnippetService(db)

	// Create
	snip, err := svc.Create(ctx, snippet.SnippetInput{
		Name:      "Test Snippet",
		Body:      "echo 'hello'",
		AutoEnter: 1,
		ScopeType: "global",
	})
	if err != nil {
		t.Fatalf("Create failed: %v", err)
	}
	if snip.ID == "" || snip.Name != "Test Snippet" {
		t.Fatalf("unexpected snippet created: %+v", snip)
	}

	// List
	list, err := svc.List(ctx)
	if err != nil {
		t.Fatalf("List failed: %v", err)
	}
	if len(list) != 1 {
		t.Fatalf("expected 1 snippet, got %d", len(list))
	}

	// Update
	updated, err := svc.Update(ctx, snip.ID, snippet.SnippetInput{
		Name:      "Updated Snippet",
		Body:      "echo 'world'",
		AutoEnter: 0,
		ScopeType: "team",
		ScopeID:   "team-1",
	})
	if err != nil {
		t.Fatalf("Update failed: %v", err)
	}
	if updated.Name != "Updated Snippet" || updated.ScopeID != "team-1" {
		t.Fatalf("unexpected updated snippet: %+v", updated)
	}

	// Delete
	if err := svc.Delete(ctx, snip.ID); err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	listAfter, err := svc.List(ctx)
	if err != nil {
		t.Fatalf("List after delete failed: %v", err)
	}
	if len(listAfter) != 0 {
		t.Fatalf("expected 0 snippets after delete, got %d", len(listAfter))
	}
}
