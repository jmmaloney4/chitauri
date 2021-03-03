package torrent

import (
	"bytes"
	"crypto/sha1"
	"fmt"
	"io"
	"log"
	"net/http"

	"github.com/cristalhq/bencode"
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

type TorrentFile struct {
	Announce    string
	InfoHash    [sha1.Size]byte
	PieceHashes [][sha1.Size]byte
	PieceLength int
	Length      int
	Name        string
}

func (i *bencodeInfo) splitPieceHashes() ([][sha1.Size]byte, error) {
	hashLen := sha1.Size // Length of SHA-1 hash
	buf := []byte(i.Pieces)
	if len(buf)%hashLen != 0 {
		err := fmt.Errorf("Received malformed pieces of length %d", len(buf))
		return nil, err
	}
	numHashes := len(buf) / hashLen
	hashes := make([][sha1.Size]byte, numHashes)

	for i := 0; i < numHashes; i++ {
		copy(hashes[i][:], buf[i*hashLen:(i+1)*hashLen])
	}
	return hashes, nil
}

func NewTorrentFile(file io.Reader) (*TorrentFile, error) {
	rv := new(TorrentFile)
	in := new(bytes.Buffer)
	in.ReadFrom(file)

	data := bencodeTorrent{}
	err := bencode.Unmarshal(in.Bytes(), &data)
	if err != nil {
		return nil, err
	}

	return rv, nil
}

func TorrentFileAtURL(url string) (*TorrentFile, error) {
	resp, err := http.Get(url)
	if err != nil {
		log.Fatal(err)
	}
	defer resp.Body.Close()

	return NewTorrentFile(resp.Body)
}
