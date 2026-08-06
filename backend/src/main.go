package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func main() {

	ctx := context.Background()
	mongo_uri := "mongodb://root:password@localhost:27017"

	mongoClient, err := startDB(ctx, mongo_uri)
	if err != nil {
		panic(err)
	}
	defer func() {
		disconnectCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		if err := mongoClient.Disconnect(disconnectCtx); err != nil {
			panic(err)
		}
		fmt.Println("Disconnected from MongoDB")
	}()

	data := newStore(mongoClient)
	indexCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	if err := data.ensureIndexing(indexCtx); err != nil {
		panic(err)
	}

	cfg := config{
		addr: ":8585",
	}
	api := api{
		config: cfg,
		store:  data,
	}

	chErr := make(chan error, 1)

	go func() {
		chErr <- api.run(api.mount())
	}()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)

	select {
	case err := <-chErr:
		log.Fatalf("server failed to start: %v", err)
	case <-stop:
		fmt.Println("Shutting down")
	}
}
