use bytes::{BufMut, BytesMut};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_bencode::{de, ser};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use snafu::{prelude::*, whatever, Whatever};
use std::{
    fs,
    io::Read,
    net::{SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
};
use tokio::net::{lookup_host, UdpSocket};
use url::Url;

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    length: i64,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: i64,
    pieces: ByteBuf,
}

#[derive(Debug, Deserialize)]
struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

impl Torrent {
    async fn announce_addr(&self) -> Result<Vec<SocketAddr>, Whatever> {
        let url = match self.announce.as_ref() {
            None => whatever!("Torrent had no announce string"),
            Some(s) => s,
        }
        .parse::<Url>()
        .whatever_context("Could not parse announce url")?;

        let port = url.port_or_known_default().unwrap_or(80);

        Ok(match url.host_str() {
            None => whatever!("Announce URL has no host"),
            Some(host) => {
                lookup_host(format!("{}:{}", host, port))
                .await
                .whatever_context("Couldn't lookup host")?
            },
        }
        .collect::<Vec<_>>())
    }
}

fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let Some(al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    // println!("private:\t{:?}", torrent.info.private);
    // println!("root hash:\t{:?}", torrent.info.root_hash);
    // println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    // println!("path:\t\t{:?}", torrent.info.path);
    // if let Some(files) = &torrent.info.files {
    //     for f in files {
    //         println!("file path:\t{:?}", f.path);
    //         println!("file length:\t{}", f.length);
    //         println!("file md5sum:\t{:?}", f.md5sum);
    //     }
    // }
}

impl Info {
    fn info_hash(&self) -> String {
        format!("{:40x}", Sha1::digest(ser::to_bytes(self).unwrap()))
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'c', long = "config", value_name = "CONFIG")]
    config: String,
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    s3: S3Config,
    peer_id: String,
    port: u32,
}

#[derive(Debug, Deserialize)]
struct S3Config {
    region: String,
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
}

#[derive(thiserror::Error, Debug)]
enum ParseConfigError {
    #[error("Couldn't open config file")]
    Open(#[from] std::io::Error),

    #[error("Couldn't parse config file")]
    SerdeYaml(#[from] serde_yaml::Error),
}

fn parse_config(path: impl AsRef<Path>) -> Result<Config, ParseConfigError> {
    let file = match fs::File::open(path.as_ref()) {
        Err(e) => return Err(ParseConfigError::Open(e)),
        Ok(f) => f,
    };

    return match serde_yaml::from_reader(file) {
        Err(e) => Err(ParseConfigError::SerdeYaml(e)),
        Ok(c) => Ok(c),
    };
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let config = match parse_config(args.config) {
        Ok(c) => c,
        Err(e) => match e {
            ParseConfigError::Open(e) => panic!("Failed to open config file: {}", e),
            ParseConfigError::SerdeYaml(e) => panic!("Failed to parse config file: {}", e),
        },
    };

    let mut file = fs::File::open(args.path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let torrent = de::from_bytes::<Torrent>(&buffer).unwrap();
    // render_torrent(&torrent);

    println!("{}", torrent.info.info_hash());

    // let mut url = Url::parse(torrent.announce.unwrap().as_str()).unwrap();
    // url.query_pairs_mut()
    //     .clear()
    //     .append_pair("info_hash", torrent.info.info_hash().as_str())
    //     .append_pair("peer_id", config.peer_id.as_str())
    //     .append_pair("port", format!("{}", config.port).as_str())
    //     .append_pair("uploaded", "0")
    //     .append_pair("downloaded", "0")
    //     .append_pair("left", "0")
    //     .append_pair("event", "started")
    //     .append_pair("compact", "0");

    // println!("{}", url);
    let port = 55555;
    let socket = UdpSocket::bind(format!("0.0.0.0:{port}").parse::<SocketAddrV4>().unwrap())
        .await
        .unwrap();

    let mut bytes = BytesMut::with_capacity(16);
    bytes.put_u64(0x41727101980);
    bytes.put_u32(0);
    bytes.put_u32(rand::random());

    println!(
        "{}",
        torrent.announce_addr().await.unwrap().first().unwrap()
    );

    // let len = socket
    //     .send_to(
    //         &bytes,
    //         torrent.annouce_addr().await.unwrap().first().unwrap(),
    //     )
    //     .await
    //     .unwrap();

    // let body = reqwest::get(url)
    // .await
    // .unwrap()
    // .text()
    // .await
    // .unwrap();

    // println!("{body}");
}
