package main

import (
	"bytes"
	"fmt"
	"log"
	"net/http"

	"github.com/jackpal/bencode-go"
)

type bencodeInfo struct {
	Pieces      string `bencode:"pieces"`
	PieceLength int    `bencode:"piece length"`
	Length      int    `bencode:"length"`
	Name        string `bencode:"name"`
}

type bencodeTorrent struct {
	Announce string      `bencode:"announce"`
	Info     bencodeInfo `bencode:"info"`
}

func main() {
	client := http.Client{
		CheckRedirect: func(r *http.Request, via []*http.Request) error {
			r.URL.Opaque = r.URL.Path
			return nil
		},
	}

	resp, err := client.Get("https://releases.ubuntu.com/20.04/ubuntu-20.04.2-live-server-amd64.iso.torrent")
	if err != nil {
		log.Fatal(err)
	}
	defer resp.Body.Close()

	in := new(bytes.Buffer)
	in.ReadFrom(resp.Body)

	data := bencodeTorrent{}
	err = bencode.Unmarshal(in, &data)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println(data)
}
