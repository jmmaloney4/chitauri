use deku::prelude::*;
use derive_builder::Builder;

use super::AnnounceEvent;
use crate::torrent::InfoHash;

const BITTORRENT_UDP_MAGIC: u64 = 0x41727101980;

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite, Builder)]
#[deku(endian = "big")]
pub(crate) struct ConnectRequest {
    #[deku(assert_eq = "BITTORRENT_UDP_MAGIC")]
    protocol_id: u64,
    #[deku(assert_eq = "Action::Connect as u32")]
    action: u32,
    transaction_id: u32,
}

enum Action {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
}

impl ConnectRequest {
    pub(crate) fn new(transaction_id: u32) -> Self {
        Self {
            protocol_id: BITTORRENT_UDP_MAGIC,
            action: Action::Connect as u32,
            transaction_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct ConnectResponse {
    #[deku(assert_eq = "Action::Connect as u32")]
    action: u32,
    transaction_id: u32,
    pub connection_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite, Builder)]
#[deku(endian = "big")]
pub(crate) struct AnnounceRequest {
    connection_id: u64,
    #[deku(assert_eq = "Action::Announce as u32")]
    action: u32,
    transaction_id: u32,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    downloaded: u64,
    left: u64,
    uploaded: u64,
    event: AnnounceEvent,
    ip: [u8; 4],
    key: u32,
    num_want: i32,
    port: u16,
}

impl AnnounceRequest {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        connection_id: u64,
        transaction_id: u32,
        info_hash: &[u8; 20],
        peer_id: &[u8; 20],
        downloaded: u64,
        left: u64,
        uploaded: u64,
        event: AnnounceEvent,
        ip: Option<&[u8; 4]>,
        key: u32,
        num_want: Option<i32>,
        port: u16,
    ) -> Self {
        Self {
            connection_id,
            action: Action::Announce as u32,
            transaction_id,
            info_hash: *info_hash,
            peer_id: *peer_id,
            downloaded,
            left,
            uploaded,
            event,
            ip: *ip.unwrap_or(&[0; 4]),
            key,
            num_want: num_want.unwrap_or(-1),
            port,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub(crate) struct AnnounceResponsePeerV4 {
    ip: [u8; 4],
    port: u16,
}

impl AnnounceResponsePeerV4 {
    pub(crate) fn new(ip: &[u8; 4], port: u16) -> Self {
        Self { ip: *ip, port }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct AnnounceResponseV4 {
    #[deku(assert_eq = "Action::Announce as u32")]
    action: u32,
    transaction_id: u32,
    interval: u32,
    leechers: u32,
    seeders: u32,
    #[deku(until = "|_| true")]
    peers: Vec<AnnounceResponsePeerV4>,
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite, Builder)]
#[deku(endian = "big")]
pub(crate) struct ScrapeRequest {
    connection_id: u64,
    #[deku(assert_eq = "Action::Scrape as u32")]
    action: u32,
    transaction_id: u32,
    #[deku(until = "|_| true")]
    info_hashes: Vec<InfoHash>,
}

impl ScrapeRequest {
    pub(crate) fn new(connection_id: u64, transaction_id: u32, info_hashes: Vec<InfoHash>) -> Self {
        Self {
            connection_id,
            action: 2,
            transaction_id,
            info_hashes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite, Builder)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub(crate) struct ScrapeResponseFile {
    seeders: u32,
    completed: u32,
    leechers: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct ScrapeResponse {
    #[deku(assert_eq = "Action::Scrape as u32")]
    action: u32,
    transaction_id: u32,
    #[deku(until = "|_| true")]
    files: Vec<ScrapeResponseFile>,
}
