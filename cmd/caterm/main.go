package main

import (
	"embed"
	"log"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"

	"caterm/internal/config"
	"caterm/internal/service/auth"
	"caterm/internal/store"
	"caterm/internal/vault"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	dbPath, err := config.GetDBPath()
	if err != nil {
		log.Fatalf("failed to get db path: %v", err)
	}

	db, err := store.Open(dbPath)
	if err != nil {
		log.Fatalf("failed to open database: %v", err)
	}
	defer db.Close()

	v := vault.NewVault()
	authService := auth.New(v, db)

	app := NewApp(authService)

	err = wails.Run(&options.App{
		Title:  "caterm",
		Width:  1024,
		Height: 768,
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 27, G: 38, B: 54, A: 1},
		OnStartup:        app.startup,
		Bind: []interface{}{
			app,
		},
	})

	if err != nil {
		println("Error:", err.Error())
	}
}
