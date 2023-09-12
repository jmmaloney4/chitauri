use log::info;
use reqwest::IntoUrl;
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
        port: Option<u16>,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        event: AnnounceEvent,
    ) -> Result<Vec<String>, reqwest::Error>;
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

impl Tracker for HTTPTracker {
    async fn get_peers(
        &self,
        info_hash: InfoHash,
        peer_id: PeerId,
        ip: Option<IpAddr>,
        port: Option<u16>,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        event: AnnounceEvent,
    ) -> Result<Vec<String>, reqwest::Error> {
        let mut url = self.url.clone();
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs
                .append_pair("info_hash", &info_hash.to_hex_string())
                .append_pair("peer_id", &peer_id.to_string())
                .append_pair("uploaded", &uploaded.to_string())
                .append_pair("downloaded", &downloaded.to_string())
                .append_pair("left", &left.to_string())
                .append_pair("event", &event.to_string());
            if port.is_some() {
                query_pairs.append_pair("port", &port.unwrap().to_string());
            }
            if ip.is_some() {
                query_pairs.append_pair("ip", &ip.unwrap().to_string());
            }
        }
        // query_pairs.finish();

        let req = reqwest::get(url).await?;
        let body = req.text().await?;
        info!("{}", body);

        Ok(Vec::new())
    }
}
