mod http;
mod udp;

use std::fmt;
use std::net::IpAddr;

use deku::prelude::*;
pub use http::HTTPTracker;
use serde::{Deserialize, Serialize};

use crate::torrent::{InfoHash, PeerId};

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

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(type = "u32", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[serde(rename_all = "lowercase")]
pub(crate) enum AnnounceEvent {
    Empty = 0,
    Completed = 1,
    Started = 2,
    Stopped = 3,
}

impl fmt::Display for AnnounceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AnnounceEvent::Empty => "empty",
            AnnounceEvent::Completed => "completed",
            AnnounceEvent::Started => "started",
            AnnounceEvent::Stopped => "stopped",
        };
        write!(f, "{}", s)
    }
}
