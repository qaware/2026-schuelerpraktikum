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
}

func (api *api) listSatellitesHandler(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSatellites(ctx)
	if err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			http.Error(w, "no satellites found", http.StatusNotFound)
			return
		}
		log.Printf("find failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, data)
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

	writeJSON(w, http.StatusOK, SpecsResponse{Specs: *data})
}

func (api *api) listNLogsHandler(w http.ResponseWriter, r *http.Request) {
	n_str := r.URL.Query().Get("amount")
	n, err := strconv.Atoi(n_str)
	if err != nil {
		http.Error(w, "amount has to be an integer", http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listNLogs(ctx, int64(n))
	if err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			http.Error(w, "no logs found", http.StatusNotFound)
			return
		}
		log.Printf("request failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, LogsResponse{Logs: data})
}
func (api *api) listSensorsHandler(w http.ResponseWriter, r *http.Request)  {
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

	writeJSON(w, http.StatusOK, SensorsResponse{Sensors: data.Sensors})
}
func (api *api) listSensorLogsHandler(w http.ResponseWriter, r *http.Request) {
	name := chi.URLParam(r, "name")
	sensor_name := chi.URLParam(r, "sensor_name")

	n_str := r.URL.Query().Get("amount")
	n, err := strconv.Atoi(n_str)
	if err != nil {
		http.Error(w, "amount has to be an integer", http.StatusBadRequest)
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), requestTimeout)
	defer cancel()

	data, err := api.store.listSensorLogs(ctx, name, sensor_name, int64(n))
	if err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			http.Error(w, "no logs found", http.StatusNotFound)
			return
		}
		log.Printf("request failed: %v", err)
		http.Error(w, "failed to query data", http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, LogsResponse{Logs: data})
}
