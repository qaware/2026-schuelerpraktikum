package main

import (
	"log"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

type api struct {
	config config
	store  *store
}
type config struct {
	addr       string
	datagenURL string
}

func (api *api) mount() http.Handler {
	r := chi.NewRouter()

	r.Use(middleware.RequestID)
	r.Use(middleware.ClientIPFromRemoteAddr) // pick one ClientIPFrom* based on your infra, see below
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

	r.Post("/data", api.ingestHandler)

	// chi matches static segments ahead of {params}, so /satellites/log and
	// /satellites/sensors win over /satellites/{name} regardless of order.
	r.Get("/satellites", api.listSatellitesHandler)
	r.Get("/satellites/log", api.listNLogsHandler)
	r.Get("/satellites/sensors", api.listAllSensorsHandler)
	r.Get("/satellites/{name}", api.listSpecsHandler)
	r.Get("/satellites/{name}/log", api.listSatelliteLogsHandler)
	r.Get("/satellites/{name}/sensors", api.listSensorsHandler)
	r.Get("/satellites/{name}/sensors/{sensor_name}", api.listSensorLogsHandler)

	// Health & Automated Integration Tests
	r.Get("/health", api.healthHandler)
	r.Get("/healthz", api.healthHandler)
	r.Post("/health/tests/run", api.runTestsHandler)

	// Admin Satellite Control Panel Routes
	r.Get("/admin/status", api.adminStatusHandler)
	r.Post("/admin/orbit", api.adminOrbitHandler)
	r.Post("/admin/anomaly", api.adminAnomalyHandler)
	r.Post("/admin/task", api.adminTaskHandler)
	r.Post("/admin/reset", api.adminResetHandler)

	return r
}

func (api *api) run(mux http.Handler) error {
	srv := &http.Server{
		Addr:    api.config.addr,
		Handler: mux,
	}

	log.Printf("Server has started at %s", api.config.addr)

	return srv.ListenAndServe()
}
