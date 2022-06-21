mod net;
mod torrent;

use bytes::{Buf, BufMut, BytesMut};
use clap::Parser;
use config::{Config, File, FileFormat};
use log::{debug, info};
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

use crate::{
    net::udp::{recv_connect_response, send_announce_request, send_connect_request},
    torrent::Torrent,
};

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

    let mut file = fs::File::open(args.path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let torrent = de::from_bytes::<Torrent>(&buffer).unwrap();

    println!("{}", torrent.info().info_hash_hex());

    let port = config.get_int("port").unwrap();
    println!("{}", port);
    let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await.unwrap();

    let addr = torrent.announce_addr().await.unwrap().next().unwrap();
    let transaction_id = send_connect_request(&socket, addr).await.unwrap();
    let connection_id = {
        let (t, c) = recv_connect_response(&socket).await.unwrap();
        if t != transaction_id {
            panic!("Transaction ID mismatch");
        }
        c
    };

    println!("{}", connection_id);

    let _ = send_announce_request(&socket, connection_id).await;
}
