package main

import (
	"bytes"
	"io"
	"net/http"
)

func (api *api) proxyToDatagen(w http.ResponseWriter, r *http.Request, path string) {
	targetURL := api.config.datagenURL + path

	var req *http.Request
	var err error

	if r.Method == http.MethodPost {
		bodyBytes, _ := io.ReadAll(r.Body)
		req, err = http.NewRequest(http.MethodPost, targetURL, bytes.NewBuffer(bodyBytes))
		req.Header.Set("Content-Type", "application/json")
	} else {
		req, err = http.NewRequest(http.MethodGet, targetURL, nil)
	}

	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
		return
	}

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		writeJSON(w, http.StatusServiceUnavailable, map[string]string{"error": "Datagen control service unavailable: " + err.Error()})
		return
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	w.Write(respBody)
}

func (api *api) adminStatusHandler(w http.ResponseWriter, r *http.Request) {
	api.proxyToDatagen(w, r, "/status")
}

func (api *api) adminOrbitHandler(w http.ResponseWriter, r *http.Request) {
	api.proxyToDatagen(w, r, "/satellites/orbit")
}

func (api *api) adminAnomalyHandler(w http.ResponseWriter, r *http.Request) {
	api.proxyToDatagen(w, r, "/satellites/anomaly")
}

func (api *api) adminTaskHandler(w http.ResponseWriter, r *http.Request) {
	api.proxyToDatagen(w, r, "/satellites/task")
}

func (api *api) adminResetHandler(w http.ResponseWriter, r *http.Request) {
	api.proxyToDatagen(w, r, "/satellites/reset")
}
