package main

import "go.mongodb.org/mongo-driver/v2/bson"

type SatelliteResponse struct {
	ID bson.ObjectID `json:"-" bson:"_id,omitempty"`
	SatelliteName string `json:"-" bson:"satellite_name"`

	SensorName string `json:"sensor_name" bson:"sensor_name"`
	Temperature float32 `json:"temperature" bson:"temperature"`
	Pressure float32 `json:"pressure" bson:"pressure"`
	Position Position `json:"position" bson:"position"`
	Time int64 `json:"time" bson:"time"`
	Info string `json:"info" bson:"info"`
	Specs Specs `json:"specs" bson:"specs"`
}
type Position struct {
	City string `json:"city" bson:"city"`
	Height float32 `json:"height" bson:"height"`
}
type Specs struct {
	Name string `json:"name" bson:"name"`
	Model string `json:"model" bson:"model"`
	LaunchDate string `json:"launch_date" bson:"launch_date"`
	Sensors []string `json:"sensors" bson:"sensors"`
	Nation string `json:"nation" bson:"nation"`
}

/* Response Types */

type SatellitesResponse struct {
	Sattellites []string `json:"sattellites"`
}
type SpecsResponse struct {
	Specs Specs `json:"specs"`
}
type LogsResponse struct {
	Logs []SatelliteResponse `json:"logs"`
}
type SensorsResponse struct {
	Sensors []string `json:"sensors"`
}
