use encoding_rs::WINDOWS_1252;
use form_urlencoded::byte_serialize;
use log::info;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use std::fs;
use std::net::IpAddr;
use url::Url;

use crate::torrent::{AnnounceEvent, InfoHash, PeerId};

enum Protocol {
    HTTP,
    UDP,
}

pub(crate) trait Tracker {
    async fn get_peers(
        &self,
        info_hash: InfoHash,
        peer_id: PeerId,
        ip: Option<IpAddr>,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        event: AnnounceEvent,
    ) -> Result<Vec<String>, reqwest::Error>;
}

// #[derive(Debug, Serialize, Deserialize)]
// pub(crate) struct HTTPTrackerResponsePeer {
//     id: Option<PeerId>,
//     ip: IpAddr,
//     port: u16,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub(crate) struct HTTPTrackerResponse {
//     interval: u32,
//     peers: Vec<HTTPTrackerResponsePeer>,
// }

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct HTTPAnnounceResponsePeer {
    id: Option<PeerId>,
    ip: String,
    port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct HTTPAnnounceResponse {
    interval: u32,
    // peers: serde_bencode::value::Value,
    peers: Vec<HTTPAnnounceResponsePeer>,
}

pub(crate) struct HTTPTracker {
    url: Url,
}

impl HTTPTracker {
    pub(crate) fn new(url: impl IntoUrl) -> Result<Self, reqwest::Error> {
        Ok(Self {
            url: url.into_url()?,
        })
    }
}

// https://github.com/servo/rust-url/issues/578
fn iso_8859_1_decode(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

fn iso_8859_1_encode(string: &str) -> Cow<[u8]> {
    string
        .chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap())
        .collect()
}

impl Tracker for HTTPTracker {
    async fn get_peers(
        &self,
        info_hash: InfoHash,
        peer_id: PeerId,
        ip: Option<IpAddr>,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        event: AnnounceEvent,
    ) -> Result<Vec<String>, reqwest::Error> {
        let mut url = self.url.clone();
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs
                .encoding_override(Some(&iso_8859_1_encode))
                .append_pair("info_hash", &iso_8859_1_decode(info_hash.as_bytes()))
                .append_pair("peer_id", &peer_id.to_string())
                .append_pair("port", &port.to_string())
                .append_pair("uploaded", &uploaded.to_string())
                .append_pair("downloaded", &downloaded.to_string())
                .append_pair("left", &left.to_string())
                .append_pair("compact", "1")
                .append_pair("no_peer_id", "0")
                .append_pair("numwant", "50");

            if ip.is_some() {
                query_pairs.append_pair("ip", &ip.unwrap().to_string());
            }
            if event != AnnounceEvent::Empty {
                query_pairs.append_pair("event", &event.to_string());
            }
        }

        info!("{}", url);
        let req = reqwest::get(url).await?;
        let body = req.text().await?;
        info!("{}", body);

        let resp = serde_bencode::from_str::<HTTPAnnounceResponse>(&body);
        info!("{:?}", resp);

        Ok(Vec::new())
    }
}
