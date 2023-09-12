use deku::prelude::*;
use generic_array::typenum::Unsigned;
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{digest::OutputSizeUser, Digest, Sha1};
use snafu::{prelude::*, whatever, Whatever};
use std::fmt;
use std::net::SocketAddr;
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
pub(crate) struct Info {
    name: String,
    #[serde(rename = "piece length")]
    piece_length: i64,
    pieces: ByteBuf,
    length: Option<i64>,
    files: Option<Vec<File>>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
// #[deku(endian = "big")]
pub(crate) struct InfoHash {
    hash: [u8; <<sha1::Sha1Core as OutputSizeUser>::OutputSize as Unsigned>::USIZE],
}

impl InfoHash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.hash
    }

    pub fn hex_string(&self) -> String {
        self.as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl fmt::Debug for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex_string())
    }
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex_string())
    }
}

impl Info {
    pub fn info_hash(&self) -> Result<InfoHash, serde_bencode::Error> {
        Ok(InfoHash {
            hash: Sha1::digest(ser::to_bytes(self)?).into(),
        })
    }
}

#[derive(Debug, Deserialize, Getters)]
pub struct Torrent {
    #[getset(get = "pub(crate)")]
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
    pub(crate) announce_list: Option<AnnounceList>,
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
    pub(crate) async fn announce_addr(&self) -> Result<impl Iterator<Item = SocketAddr>, Whatever> {
        let url = match self.announce.as_ref() {
            None => whatever!("Torrent had no announce string"),
            Some(s) => s,
        }
        .parse::<Url>()
        .whatever_context("Could not parse announce url")?;

        let port = url.port_or_known_default().unwrap_or(80);

        Ok(match url.host_str() {
            None => whatever!("Announce URL has no host"),
            Some(host) => tokio::net::lookup_host(format!("{host}:{port}"))
                .await
                .whatever_context("Couldn't lookup host")?,
        })
    }
}

/// See: http://www.bittorrent.org/beps/bep_0012.html
#[derive(Debug)]
pub(crate) struct AnnounceList {
    list: Vec<Vec<Url>>,
}

impl<'de> Deserialize<'de> for AnnounceList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let list = Vec::<Vec<String>>::deserialize(deserializer)?
            .into_iter()
            .map(|urls| {
                urls.into_iter()
                    .map(|url| url.parse::<Url>())
                    .collect::<Result<Vec<Url>, _>>()
            })
            .collect::<Result<Vec<Vec<Url>>, _>>()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(AnnounceList { list })
    }
}
