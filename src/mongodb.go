package main

import (
	"context"
	"fmt"
	"reflect"
	"time"

	"go.mongodb.org/mongo-driver/v2/bson"
	"go.mongodb.org/mongo-driver/v2/mongo"
	"go.mongodb.org/mongo-driver/v2/mongo/options"
)

func startDB(ctx context.Context, uri string) (*mongo.Client, error) {
	client, err := mongo.Connect(options.Client().ApplyURI(uri))
	if err != nil {
		return nil, err
	}
	
	pingCtx, cancel := context.WithTimeout(ctx, time.Second * 10)
	defer cancel()
	if err := client.Ping(pingCtx, nil); err != nil {
		return nil, err
	}

	fmt.Println("Connected to MongoDB")
	return client, nil
}

type store struct {
	collection *mongo.Collection
}

func newStore(client *mongo.Client) *store {
	collection := client.Database("pizza_party").Collection("satellite_data") 
	return &store{collection: collection}
}

func (store *store) ensureIndexing(ctx context.Context) error {
	_, err := store.collection.Indexes().CreateMany(ctx, []mongo.IndexModel{
		{
			Keys: bson.D{
				{Key: "satellite_name", Value: 1},
				{Key: "sensor_name", Value: 1},
				{Key: "time", Value: -1},
			},
		},
		{
			Keys: bson.D{
				{Key: "time", Value: -1},
			},
		},
	})
	return err
}

func (store *store) insert(ctx context.Context, data SatelliteResponse) error {
	_, err := store.collection.InsertOne(ctx, &data)
	return err
}

func (store *store) listSatellites(ctx context.Context) ([]string, error) {
	var satellites []string

	err := store.collection.Distinct(ctx, "satellite_name", bson.D{}).Decode(&satellites)
	if err != nil {
		return nil, err
	}
	return satellites, nil
}

func (store *store) listSpecs(ctx context.Context, satellite_name string) (*Specs, error) {
	opts := options.FindOne().
	SetSort(bson.D{{Key: "time", Value: -1}}).
	SetProjection(bson.D{{Key: "specs", Value: 1}, {Key: "_id", Value: 0}})

	var result struct {
		Specs Specs `bson:"specs"`
	}

	err := store.collection.FindOne(ctx, bson.D{{Key: "satellite_name", Value: satellite_name}}, opts).Decode(&result)
	if err != nil {
		return nil, err
	}
	if reflect.DeepEqual(result.Specs, Specs{}) {
		return nil, fmt.Errorf("%q has no specs in the latest log", satellite_name)
	}
	return &result.Specs, nil
}

func (store *store) listNLogs(ctx context.Context, n int64) ([]SatelliteResponse, error) {
	opts := options.Find().
	SetSort(bson.D{{Key: "time", Value: -1}}).
	SetLimit(n)

	cursor, err := store.collection.Find(ctx, bson.D{}, opts)
	if err != nil {
		return nil, err
	}
	defer cursor.Close(ctx)

	var logs []SatelliteResponse

	if err := cursor.All(ctx, &logs); err != nil {
		return nil, err
	}
	return logs, nil
}
func (store *store) listSensorLogs(ctx context.Context, satellite_name string, sensor_name string, n int64) ([]SatelliteResponse, error) {
	opts := options.Find().
	SetSort(bson.D{{Key: "time", Value: -1}}).
	SetLimit(n)

	filter := bson.D{
		{Key: "satellite_name", Value: satellite_name},
		{Key: "sensor_name", Value: sensor_name},
	}

	cursor, err := store.collection.Find(ctx, filter, opts)
	if err != nil {
		return nil, err
	}
	defer cursor.Close(ctx)

	var logs []SatelliteResponse

	if err := cursor.All(ctx, &logs); err != nil {
		return nil, err
	}
	return logs, nil
}
