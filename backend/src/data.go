package main

import (
	"context"
	"errors"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/go-chi/chi/v5"
	"go.mongodb.org/mongo-driver/v2/mongo"
)

var requestTimeout = time.Second * 5

const (
	defaultAmount = 100
	maxAmount     = 1000
)

// parseAmount reads the optional ?amount= query parameter. Absent means the
// contract default; anything above maxAmount is clamped instead of rejected so
// a greedy client gets fewer rows rather than an error.
func parseAmount(r *http.Request) (int64, error) {
	raw := r.URL.Query().Get("amount")
	if raw == "" {
		return defaultAmount, nil
	}

	n, err := strconv.Atoi(raw)
	if err != nil {
		return 0, errors.New("amount has to be an integer")
	}
	if n < 1 {
		return 0, errors.New("amount has to be at least 1")
	}
	if n > maxAmount {
		n = maxAmount
	}
	return int64(n), nil
}

// writeLogs flips the newest-first order Mongo returns back into the ascending
// order the API contract promises, then wraps it in the {amount, data} envelope.
func writeLogs(w http.ResponseWriter, logs []SatelliteResponse) {
	if logs == nil {
		logs = []SatelliteResponse{}
	}
	for i, j := 0, len(logs)-1; i < j; i, j = i+1, j-1 {
		logs[i], logs[j] = logs[j], logs[i]
	}
	writeJSON(w, http.StatusOK, LogsResponse{Amount: len(logs), Data: logs})
}

func (api *api) ingestHandler(w http.ResponseWriter, r *http.Request) {
	data, err := readAndDecode(r.Body)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	if err := api.store.insert(ctx, data); err != nil {
		fmt.Printf("insert failed: %v", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusCreated)
}

func (api *api) listSatellitesHandler(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSatellites(ctx)
	if err != nil {
		log.Printf("find failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}
	if data == nil {
		data = []string{}
	}

	writeJSON(w, http.StatusOK, NamesResponse{Names: data})
}

func (api *api) listAllSensorsHandler(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listAllSensors(ctx)
	if err != nil {
		log.Printf("find failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}
	if data == nil {
		data = []string{}
	}

	writeJSON(w, http.StatusOK, SensorsResponse{SensorNames: data})
}

func (api *api) listSpecsHandler(w http.ResponseWriter, r *http.Request) {
	name := chi.URLParam(r, "name")

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSpecs(ctx, name)
	if err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			http.Error(w, "no specs found", http.StatusNotFound)
			return
		}
		log.Printf("find failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, SatelliteDetailResponse{
		Name:       data.Name,
		Model:      data.Model,
		LaunchDate: data.LaunchDate,
		Sensors:    data.Sensors,
		Nation:     data.Nation,
	})
}

func (api *api) listNLogsHandler(w http.ResponseWriter, r *http.Request) {
	n, err := parseAmount(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listNLogs(ctx, n)
	if err != nil {
		log.Printf("request failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeLogs(w, data)
}

func (api *api) listSatelliteLogsHandler(w http.ResponseWriter, r *http.Request) {
	name := chi.URLParam(r, "name")

	n, err := parseAmount(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSatelliteLogs(ctx, name, n)
	if err != nil {
		log.Printf("request failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeLogs(w, data)
}

func (api *api) listSensorsHandler(w http.ResponseWriter, r *http.Request) {
	name := chi.URLParam(r, "name")

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSpecs(ctx, name)
	if err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			http.Error(w, "no specs found", http.StatusNotFound)
			return
		}
		log.Printf("find failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	sensors := data.Sensors
	if sensors == nil {
		sensors = []string{}
	}

	writeJSON(w, http.StatusOK, SensorsResponse{SensorNames: sensors})
}

func (api *api) listSensorLogsHandler(w http.ResponseWriter, r *http.Request) {
	name := chi.URLParam(r, "name")
	sensor_name := chi.URLParam(r, "sensor_name")

	n, err := parseAmount(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSensorLogs(ctx, name, sensor_name, n)
	if err != nil {
		log.Printf("request failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeLogs(w, data)
}
