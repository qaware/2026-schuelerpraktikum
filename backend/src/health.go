package main

import (
	"context"
	"fmt"
	"net/http"
	"runtime"
	"time"
)

var startTime = time.Now()

func (api *api) healthHandler(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()

	components := make(map[string]ComponentHealth)
	overallStatus := "HEALTHY"

	// 1. Backend Runtime
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	components["backend"] = ComponentHealth{
		Status: "ONLINE",
		Details: map[string]interface{}{
			"goroutines": runtime.NumGoroutine(),
			"alloc_mb":   m.Alloc / 1024 / 1024,
			"sys_mb":     m.Sys / 1024 / 1024,
			"gc_runs":    m.NumGC,
		},
		LatencyMs: 0,
	}

	// 2. MongoDB Check
	mongoStart := time.Now()
	err := api.store.client.Ping(ctx, nil)
	mongoLatency := time.Since(mongoStart).Milliseconds()
	if err != nil {
		overallStatus = "DEGRADED"
		components["mongodb"] = ComponentHealth{
			Status:    "OFFLINE",
			Details:   map[string]interface{}{"error": err.Error()},
			LatencyMs: mongoLatency,
		}
	} else {
		count, _ := api.store.collection.CountDocuments(ctx, map[string]interface{}{})
		components["mongodb"] = ComponentHealth{
			Status: "ONLINE",
			Details: map[string]interface{}{
				"documents_count": count,
				"database":        "pizza_party",
				"collection":      "satellite_data",
			},
			LatencyMs: mongoLatency,
		}
	}

	// 3. Datagen Service Check
	datagenStart := time.Now()
	dgResp, dgErr := http.Get(api.config.datagenURL + "/health")
	dgLatency := time.Since(datagenStart).Milliseconds()
	if dgErr != nil || dgResp.StatusCode != 200 {
		components["datagen"] = ComponentHealth{
			Status:    "OFFLINE",
			Details:   map[string]interface{}{"url": api.config.datagenURL},
			LatencyMs: dgLatency,
		}
		if overallStatus == "HEALTHY" {
			overallStatus = "DEGRADED"
		}
	} else {
		dgResp.Body.Close()
		components["datagen"] = ComponentHealth{
			Status:    "ONLINE",
			Details:   map[string]interface{}{"url": api.config.datagenURL},
			LatencyMs: dgLatency,
		}
	}

	resp := SystemHealthResponse{
		Status:     overallStatus,
		UptimeSec:  int64(time.Since(startTime).Seconds()),
		Timestamp:  time.Now().Unix(),
		Components: components,
	}

	writeJSON(w, http.StatusOK, resp)
}

func (api *api) runTestsHandler(w http.ResponseWriter, r *http.Request) {
	start := time.Now()
	steps := []TestStepResult{}

	// Test 1: DB Ping
	t1Start := time.Now()
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()
	err := api.store.client.Ping(ctx, nil)
	if err == nil {
		steps = append(steps, TestStepResult{
			Name:     "Database Connection Ping",
			Passed:   true,
			Duration: time.Since(t1Start).Milliseconds(),
			Message:  "MongoDB cluster responded successfully to PING.",
		})
	} else {
		steps = append(steps, TestStepResult{
			Name:     "Database Connection Ping",
			Passed:   false,
			Duration: time.Since(t1Start).Milliseconds(),
			Message:  "MongoDB PING failed: " + err.Error(),
		})
	}

	// Test 2: Ingest & Query Roundtrip
	t2Start := time.Now()
	tempVal := float32(42.5)
	pressVal := float32(3.14)
	dummyRecord := SatelliteResponse{
		SensorName:  "test_probe_health",
		Temperature: &tempVal,
		Pressure:    &pressVal,
		Position: Position{
			City:   "TestLab",
			Height: 999.9,
		},
		Specs: Specs{
			Name:       "TEST_SAT",
			Model:      "TestModel-X",
			LaunchDate: "01.01.2026",
			Sensors:    []string{"test_probe_health"},
			Nation:     "QAware",
		},
		Time: time.Now().Unix(),
	}

	err = api.store.insert(ctx, dummyRecord)
	if err == nil {
		steps = append(steps, TestStepResult{
			Name:     "Telemetry Ingestion & Store",
			Passed:   true,
			Duration: time.Since(t2Start).Milliseconds(),
			Message:  "Test record inserted into database successfully.",
		})
	} else {
		steps = append(steps, TestStepResult{
			Name:     "Telemetry Ingestion & Store",
			Passed:   false,
			Duration: time.Since(t2Start).Milliseconds(),
			Message:  "Insert failed: " + err.Error(),
		})
	}

	// Test 3: Datagen API Connectivity
	t3Start := time.Now()
	dgResp, dgErr := http.Get(api.config.datagenURL + "/status")
	if dgErr == nil && dgResp.StatusCode == 200 {
		dgResp.Body.Close()
		steps = append(steps, TestStepResult{
			Name:     "Datagen Control API Reachability",
			Passed:   true,
			Duration: time.Since(t3Start).Milliseconds(),
			Message:  "Datagen Control API on port 8090 is active.",
		})
	} else {
		msg := "Datagen API unreachable"
		if dgErr != nil {
			msg = dgErr.Error()
		}
		steps = append(steps, TestStepResult{
			Name:     "Datagen Control API Reachability",
			Passed:   false,
			Duration: time.Since(t3Start).Milliseconds(),
			Message:  msg,
		})
	}

	// Test 4: Schema Validation Test
	t4Start := time.Now()
	sats, satErr := api.store.listSatellites(ctx)
	if satErr == nil {
		steps = append(steps, TestStepResult{
			Name:     "Satellite Schema & Catalog Query",
			Passed:   true,
			Duration: time.Since(t4Start).Milliseconds(),
			Message:  fmt.Sprintf("Queried satellite list successfully (%d registered).", len(sats)),
		})
	} else {
		steps = append(steps, TestStepResult{
			Name:     "Satellite Schema & Catalog Query",
			Passed:   false,
			Duration: time.Since(t4Start).Milliseconds(),
			Message:  "Catalog query failed: " + satErr.Error(),
		})
	}

	passedCount := 0
	failedCount := 0
	for _, step := range steps {
		if step.Passed {
			passedCount++
		} else {
			failedCount++
		}
	}

	res := TestSuiteResult{
		Total:       len(steps),
		PassedCount: passedCount,
		FailedCount: failedCount,
		DurationMs:  time.Since(start).Milliseconds(),
		Steps:       steps,
	}

	writeJSON(w, http.StatusOK, res)
}
