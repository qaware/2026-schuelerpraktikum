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
type Position struct {
	City   string  `json:"city" bson:"city"`
	Height float32 `json:"height" bson:"height"`
}
type Specs struct {
	Name       string   `json:"name" bson:"name"`
	Model      string   `json:"model" bson:"model"`
	LaunchDate string   `json:"launch_date" bson:"launch_date"`
	Sensors    []string `json:"sensors" bson:"sensors"`
	Nation     string   `json:"nation" bson:"nation"`
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
	Name       string   `json:"name"`
	Model      string   `json:"model"`
	LaunchDate string   `json:"launchdate"`
	Sensors    []string `json:"sensors"`
	Nation     string   `json:"nation"`
}
