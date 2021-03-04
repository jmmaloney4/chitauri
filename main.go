package main

import (
	"fmt"
	"log"

	"github.com/jmmaloney4/chitauri/config"
	"github.com/jmmaloney4/chitauri/torrent"
	"github.com/minio/minio-go"
	"github.com/spf13/viper"
)

func main() {
	viper.SetConfigName("config")          // name of config file (without extension)
	viper.SetConfigType("yaml")            // REQUIRED if the config file does not have the extension in the name
	viper.AddConfigPath("$HOME/.chitauri") // call multiple times to add many search paths
	viper.AddConfigPath(".")

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			// Config file not found; ignore error if desired
			log.Println("No config file found.")
		} else {
			// Config file was found but another error was produced
			log.Fatal(err)
		}
	}

	cfg := new(config.Config)

	err := viper.Unmarshal(cfg)
	if err != nil {
		log.Fatal(err)
	}

	var clients []*minio.Client
	for _, e := range cfg.Endpoints {
		c, err := e.ToMinioClient()
		if err != nil {
			log.Fatal(err)
		}
		clients = append(clients, c)
	}

	for i, e := range cfg.Endpoints {
		if e.Name == cfg.Data.Endpoint {
			exists, err := clients[i].BucketExists(cfg.Data.Bucket)
			if err != nil {
				log.Fatal(err)
			}
			if !exists {
				fmt.Printf("Bucket %s does not exist at endpoint %s, creating it.", cfg.Data.Bucket, cfg.Data.Endpoint)
				clients[i].MakeBucket(cfg.Data.Bucket, "us-east-1")
			}
		}
	}

	torrent, err := torrent.TorrentFileAtURL("https://releases.ubuntu.com/20.04/ubuntu-20.04.2-live-server-amd64.iso.torrent")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%s: %d pieces\n", torrent.Name, len(torrent.PieceHashes))
}
