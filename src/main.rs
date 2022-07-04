mod net;
mod torrent;

use bytes::{Buf, BufMut, BytesMut};
use clap::Parser;
use config::{Config, File, FileFormat};
use hyper::client::connect::Connect;
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

use deku::prelude::*;

use crate::torrent::Torrent;

// use crate::{
//     net::udp::{
//         packet::{announce_response, connect_request},
//         recv_connect_response,
//     },
//     torrent::Torrent,
// };

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

    // println!("{}", torrent.info().info_hash_hex());

    let port: u16 = config.get_int("port").unwrap().try_into().unwrap();
    let peer_id = {
        let mut buf = [0_u8; 20];
        buf.copy_from_slice(&config.get_string("peer_id").unwrap().as_bytes()[0..20]);
        buf
    };
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await.unwrap();

    let addr = torrent.announce_addr().await.unwrap().next().unwrap();
    println!("{}", addr);
    let tx_id = rand::random();
    println!("{}", tx_id);
    socket
        .send_to(
            ConnectRequest::new(tx_id).to_bytes().unwrap().as_ref(),
            addr,
        )
        .await
        .unwrap();
    println!("Sent connect request");

    let mut buf = [0_u8; u16::MAX as usize];
    let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
    let connect_response = ConnectResponse::from_bytes((&buf[0..len], 0)).unwrap();
    println!("{:?}", connect_response.1);

    // let (_, connection_id) = recv_connect_response(&socket).await.unwrap();
    // println!("{}", connection_id);

    // socket
    //     .send_to(
    //         &announce_response(
    //             connection_id,
    //             rand::random(),
    //             &torrent.info().info_hash(),
    //             &peer_id,
    //             0,
    //             0,
    //             0,
    //             None,
    //             Option::<std::net::Ipv4Addr>::None,
    //             rand::random(),
    //             None,
    //             port,
    //         ),
    //         addr,
    //     )
    //     .await
    //     .unwrap();

    // let mut buf = BytesMut::new();
    // buf.put_bytes(0, 4096);
    // let (l, addr) = socket.recv_from(buf.as_mut()).await.unwrap();
    // println!("{} {}", l, addr);
    // println!("{:?}", buf.as_ref());

    // let _ = send_announce_request(&socket, connection_id).await;
}

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct ConnectRequest {
    protocol_id: u64,
    action: u32,
    transaction_id: u32,
}

impl ConnectRequest {
    pub(crate) fn new(transaction_id: u32) -> Self {
        Self {
            protocol_id: 0x41727101980,
            action: 0,
            transaction_id,
        }
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct ConnectResponse {
    action: u32,
    transaction_id: u32,
    connection_id: u64,
}

impl ConnectResponse {
    pub(crate) fn new(transaction_id: u32, connection_id: u64) -> Self {
        Self {
            action: 0,
            transaction_id,
            connection_id,
        }
    }
}
