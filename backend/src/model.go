package main

import "go.mongodb.org/mongo-driver/v2/bson"

// SatelliteResponse is both the ingest payload and the log entry returned to
// clients. Temperature and Pressure are pointers because a tank only carries
// one of the two probes: a plain float32 would turn a missing reading into 0.
type SatelliteResponse struct {
	ID            bson.ObjectID `json:"-" bson:"_id,omitempty"`
	SatelliteName string        `json:"-" bson:"satellite_name"`

	SensorName  string   `json:"sensor_name" bson:"sensor_name"`
	Temperature *float32 `json:"temperature" bson:"temperature"`
	Pressure    *float32 `json:"pressure" bson:"pressure"`
	Position    Position `json:"position" bson:"position"`
	Specs       Specs    `json:"specs" bson:"specs"`
	// The wire name is "timestamp"; the bson name stays "time" so the existing
	// indexes keep working.
	Time int64 `json:"timestamp" bson:"time"`
}

// Position is the sub-satellite point: the spot on the surface the satellite
// is directly above, plus its altitude. Latitude and Longitude are what let a
// client place the satellite on a globe -- without them, City alone only
// narrows it down to one of a handful of ground stations.
type Position struct {
	City      string  `json:"city" bson:"city"`
	Height    float32 `json:"height" bson:"height"`
	Latitude  float32 `json:"latitude" bson:"latitude"`
	Longitude float32 `json:"longitude" bson:"longitude"`
}
type Specs struct {
	Name       string   `json:"name" bson:"name"`
	Model      string   `json:"model" bson:"model"`
	LaunchDate string   `json:"launch_date" bson:"launch_date"`
	Sensors    []string `json:"sensors" bson:"sensors"`
	Nation     string   `json:"nation" bson:"nation"`
	// Orbital inclination in degrees.
	Inclination float32 `json:"inclination" bson:"inclination"`
}

/* Response Types */

type NamesResponse struct {
	Names []string `json:"names"`
}
type SensorsResponse struct {
	SensorNames []string `json:"sensor_names"`
}
type LogsResponse struct {
	Amount int                 `json:"amount"`
	Data   []SatelliteResponse `json:"data"`
}
type SatelliteDetailResponse struct {
	Name        string   `json:"name"`
	Model       string   `json:"model"`
	LaunchDate  string   `json:"launchdate"`
	Sensors     []string `json:"sensors"`
	Nation      string   `json:"nation"`
	Inclination float32  `json:"inclination"`
}

/* Health & Diagnostic Types */

type ComponentHealth struct {
	Status    string                 `json:"status"`
	Details   map[string]interface{} `json:"details,omitempty"`
	LatencyMs int64                  `json:"latency_ms"`
}

type SystemHealthResponse struct {
	Status     string                     `json:"status"`
	UptimeSec  int64                      `json:"uptime_sec"`
	Timestamp  int64                      `json:"timestamp"`
	Components map[string]ComponentHealth `json:"components"`
}

type TestStepResult struct {
	Name     string `json:"name"`
	Passed   bool   `json:"passed"`
	Duration int64  `json:"duration_ms"`
	Message  string `json:"message"`
}

type TestSuiteResult struct {
	Total       int              `json:"total"`
	PassedCount int              `json:"passed_count"`
	FailedCount int              `json:"failed_count"`
	DurationMs  int64            `json:"duration_ms"`
	Steps       []TestStepResult `json:"steps"`
}
