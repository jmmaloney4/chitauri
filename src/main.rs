#![feature(async_fn_in_trait)]

// mod net;
mod torrent;
mod tracker;

use clap::Parser;
use config::{Config, File, FileFormat};

use log::{debug, info};
// use net::udp::AnnounceRequest;

use serde_bencode::{de, ser};
use std::{fs, io::Read, path::PathBuf};
use tokio::net::UdpSocket;

use deku::prelude::*;

use crate::torrent::{PeerId, Torrent};
use crate::tracker::HTTPTracker;
use crate::tracker::Tracker;

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
            eprintln!("Couldn't parse config file: {e}");
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

    info!("{:?}", torrent.info());

    info!(
        "trackers: {:#?}",
        torrent.info().info_hash().unwrap().to_hex_string()
    );

    let peerid = PeerId::new();
    let tracker =
        HTTPTracker::new(torrent.announce_list.as_ref().unwrap().list[0][0].as_str()).unwrap();
    let peers = tracker
        .get_peers(
            torrent.info().info_hash().unwrap(),
            peerid,
            None,
            None,
            0,
            0,
            0,
            torrent::AnnounceEvent::Empty,
        )
        .await
        .unwrap();

    // println!("{}", torrent.info().info_hash_hex());

    // let port: u16 = config.get_int("port").unwrap().try_into().unwrap();
    // let peer_id = {
    //     let mut buf = [0_u8; 20];
    //     buf.copy_from_slice(&config.get_string("peer_id").unwrap().as_bytes()[0..20]);
    //     buf
    // };

    // let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await.unwrap();

    // let addr = torrent.announce_addr().await.unwrap().next().unwrap();
    // println!("{addr}");
    // let tx_id = rand::random();
    // println!("{tx_id}");
    // socket
    //     .send_to(
    //         ConnectRequest::new(tx_id).to_bytes().unwrap().as_ref(),
    //         addr,
    //     )
    //     .await
    //     .unwrap();
    // println!("Sent connect request");

    // let mut buf = [0_u8; u16::MAX as usize];
    // let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
    // let (_, connect_response) = ConnectResponse::from_bytes((&buf[0..len], 0)).unwrap();
    // println!("{connect_response:?}");

    // socket
    //     .send_to(
    //         AnnounceRequest::new(
    //             connect_response.connection_id,
    //             rand::random(),
    //             &torrent.info().info_hash(),
    //             &peer_id,
    //             0,
    //             0,
    //             0,
    //             net::udp::AnnounceEvent::None,
    //             None,
    //             rand::random(),
    //             None,
    //             port,
    //         )
    //         .to_bytes()
    //         .unwrap()
    //         .as_ref(),
    //         addr,
    //     )
    //     .await
    //     .unwrap();

    // let (len, _addr) = socket.recv_from(&mut buf).await.unwrap();
    // let (_, announce_response) = AnnounceResponseV4::from_bytes((&buf[0..len], 0)).unwrap();
    // println!("{announce_response:?}");

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
