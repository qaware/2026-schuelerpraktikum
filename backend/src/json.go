package main

import (
	"encoding/json"
	"errors"
	"io"
	"net/http"
)

func readAndDecode(r io.Reader) (SatelliteResponse, error) {
	var data SatelliteResponse

	decoder := json.NewDecoder(r)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&data); err != nil {
		return SatelliteResponse{}, err
	}

	if data.Specs.Name == "" {
		return SatelliteResponse{}, errors.New("satellite name is required")
	}
	if data.SensorName == "" {
		return SatelliteResponse{}, errors.New("sensor name is required")
	}
	if data.Time == 0 {
		return SatelliteResponse{}, errors.New("timestamp is required")
	}
	if data.Temperature == nil && data.Pressure == nil {
		return SatelliteResponse{}, errors.New("at least one of temperature or pressure is required")
	}

	data.SatelliteName = data.Specs.Name
	return data, nil
}
func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}
