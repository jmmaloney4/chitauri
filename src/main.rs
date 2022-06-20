mod net;
mod torrent;

use bytes::{Buf, BufMut, BytesMut};
use clap::Parser;
use config::{Config, File, FileFormat};
use log::{info, debug};
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

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'c', long = "config", value_name = "CONFIG")]
    config: String,
    path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    simple_logger::SimpleLogger::new().init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let config = match Config::builder()
        .add_source(File::new(args.config.as_str(), FileFormat::Yaml))
        .build()
    {
        Err(e) => {
            eprintln!("Couldn't parse config file: {}", e);
            std::process::exit(1);
        }
        Ok(c) => c,
    };

    info!("Successfully loaded config file {}", args.config);
    debug!("{:#?}", config);

    // println!("{:?}", config.get_int("port").unwrap());

    // let config = match parse_config(args.config) {
    //     Ok(c) => c,
    //     Err(e) => match e {
    //         ParseConfigError::Open(e) => panic!("Failed to open config file: {}", e),
    //         ParseConfigError::SerdeYaml(e) => panic!("Failed to parse config file: {}", e),
    //     },
    // };

    // let mut file = fs::File::open(args.path).unwrap();
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer).unwrap();
    // let torrent = de::from_bytes::<Torrent>(&buffer).unwrap();
    // // render_torrent(&torrent);

    // println!("{}", torrent.info.info_hash());

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
    // let port = 55555;
    // let socket = UdpSocket::bind(format!("0.0.0.0:{port}").parse::<SocketAddrV4>().unwrap())
    //     .await
    //     .unwrap();

    // let mut bytes = BytesMut::with_capacity(16);
    // bytes.put_u64(0x41727101980);
    // bytes.put_u32(0);
    // bytes.put_u32(rand::random());

    // println!(
    //     "{}",
    //     torrent.announce_addr().await.unwrap().first().unwrap()
    // );

    // let len = socket
    //     .send_to(
    //         &bytes,
    //         torrent.announce_addr().await.unwrap().first().unwrap(),
    //     )
    //     .await
    //     .unwrap();

    // let mut buf = BytesMut::new();
    // buf.put_bytes(0, 16);

    // let (l, addr) = socket.recv_from(buf.as_mut()).await.unwrap();
    // println!("{}, {}, {:?}", addr, l, buf);

    // if buf.get_u32() != 0 {
    //     whatever!("Nonzero ", );
    // }

    // println!("{} {} {}", buf.get_u32(), buf.get_u32(), buf.get_u64());

    // let body = reqwest::get(url)
    // .await
    // .unwrap()
    // .text()
    // .await
    // .unwrap();

    // println!("{body}");
}
