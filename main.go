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

	config := new(config.Config)

	err := viper.Unmarshal(config)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(config)

	minioClient, err := minio.New(config.Endpoints[0].Url, config.Endpoints[0].AccessKey, config.Endpoints[0].SecretKey, config.Endpoints[0].Ssl)
	if err != nil {
		log.Fatalln(err)
	}
	fmt.Println(minioClient.ListBuckets())

	torrent, err := torrent.TorrentFileAtURL("https://releases.ubuntu.com/20.04/ubuntu-20.04.2-live-server-amd64.iso.torrent")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%s: %d pieces\n", torrent.Name, len(torrent.PieceHashes))
}
