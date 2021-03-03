package main

import (
	"fmt"
	"log"

	"github.com/jmmaloney4/chitauri/torrent"
)

func main() {
	torrent, err := torrent.TorrentFileAtURL("https://releases.ubuntu.com/20.04/ubuntu-20.04.2-live-server-amd64.iso.torrent")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%s: %d pieces\n", torrent.Name, len(torrent.PieceHashes))
}
