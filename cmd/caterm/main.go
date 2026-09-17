package main

import (
	"embed"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"

	"caterm/internal/config"
	"caterm/internal/service/auth"
	"caterm/internal/service/host"
	"caterm/internal/store"
	"caterm/internal/vault"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	dbPath := config.DataPath()

	db, err := store.Open(dbPath)
	if err != nil {
		println("failed to open database:", err.Error())
		return
	}
	defer db.Close()

	v := vault.NewVault()
	authService := auth.New(v, db)
	groupService := host.NewGroupService(db)
	hostService := host.NewHostService(db, v)

	app := NewApp(authService, groupService, hostService)

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
