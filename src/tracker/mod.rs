use encoding_rs::WINDOWS_1252;
use form_urlencoded::byte_serialize;
use log::info;
use reqwest::IntoUrl;
use serde::{de, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use std::fs;
use std::net::IpAddr;
use std::str::FromStr;
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

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub(crate) struct HTTPAnnounceResponsePeer {
    id: Option<PeerId>,
    #[serde(deserialize_with = "deserialize_ipaddr")]
    ip: IpAddr,
    port: u16,
}

fn deserialize_ipaddr<'de, D>(deserializer: D) -> Result<IpAddr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    println!("PARSING IP: {}", s);
    match IpAddr::from_str(&s) {
        Ok(ip) => Ok(ip),
        Err(e) => Err(serde::de::Error::custom(format!(
            "failed to parse ip address: {}",
            e
        ))),
    }
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub(crate) struct HTTPAnnounceResponse {
    interval: u32,
    // peers: serde_bencode::value::Value,
    #[serde(deserialize_with = "deserialize_peers")]
    peers: Vec<HTTPAnnounceResponsePeer>,
}

fn deserialize_peers<'de, D>(deserializer: D) -> Result<Vec<HTTPAnnounceResponsePeer>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // println!("DESERIALIZING PEERS");
    // let x = <Vec<HTTPAnnounceResponsePeer>>::deserialize(deserializer)?;

    // println!("{:?}", x);
    // Ok(x)

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PeerType {
        NonCompact(Vec<HTTPAnnounceResponsePeer>),
        Compact(ByteBuf),
    }

    let x = match PeerType::deserialize(deserializer)? {
        PeerType::NonCompact(peers) => Ok(peers),
        PeerType::Compact(bytes) => deserialize_compact_peers(&bytes).map_err(de::Error::custom),
    };
    println!("{:?}", x);
    x
}

fn deserialize_compact_peers(
    bytes: &[u8],
) -> Result<Vec<HTTPAnnounceResponsePeer>, serde_bencode::Error> {
    println!("DESERIALIZING COMPACT PEERS");
    if bytes.len() % 6 != 0 {
        return Err(serde_bencode::Error::Custom(format!(
            "invalid compact peer list length: {}",
            bytes.len()
        )));
    }
    let mut peers = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let ip = IpAddr::from([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let port = u16::from_be_bytes([bytes[i + 4], bytes[i + 5]]);
        peers.push(HTTPAnnounceResponsePeer { id: None, ip, port });
        i += 6;
    }
    println!("{:?}", peers);
    Ok(peers)
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
        let body = req.bytes().await?;
        // info!("{}", String::from_utf8_lossy(&body));
        println!("{:?}", body);

        let resp = serde_bencode::from_bytes::<HTTPAnnounceResponse>(&body);
        info!("{:?}", resp);

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::tracker::{HTTPAnnounceResponse, HTTPAnnounceResponsePeer};
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;

    #[test]
    fn deserialize_http_response_non_compact() {
        let str = "d8:completei113e10:incompletei3e8:intervali1800e5:peersld2:ip14:185.125.190.594:porti6888eed2:ip39:2a02:1210:4831:9700:ba27:ebff:fe91:60cd4:porti51316eed2:ip22:2620:1d5:ffd:1702::2134:porti51413eeee";
        let resp = serde_bencode::from_str::<HTTPAnnounceResponse>(str);
        assert!(resp.is_ok());
        assert_eq!(
            resp.unwrap(),
            HTTPAnnounceResponse {
                interval: 1800,
                peers: vec![
                    HTTPAnnounceResponsePeer {
                        id: None,
                        ip: IpAddr::V4(Ipv4Addr::new(185, 125, 190, 59)),
                        port: 6888,
                    },
                    HTTPAnnounceResponsePeer {
                        id: None,
                        ip: IpAddr::from_str("2a02:1210:4831:9700:ba27:ebff:fe91:60cd").unwrap(),
                        port: 51316,
                    },
                    HTTPAnnounceResponsePeer {
                        id: None,
                        ip: IpAddr::from_str("2620:1d5:ffd:1702::213").unwrap(),
                        port: 51413,
                    },
                ],
            }
        );
    }
    #[test]
    fn deserialize_http_response_compact() {
        let bytes = b"d8:completei12e10:incompletei1e8:intervali1800e5:peers6:\xb9}\xbe;\x1b\x1ee";
        let resp = serde_bencode::from_bytes::<HTTPAnnounceResponse>(bytes);
        assert!(resp.is_ok());
        assert_eq!(
            resp.unwrap(),
            HTTPAnnounceResponse {
                interval: 1800,
                peers: vec![HTTPAnnounceResponsePeer {
                    id: None,
                    ip: IpAddr::V4(Ipv4Addr::new(185, 125, 190, 59)),
                    port: 6942
                }]
            }
        )
    }
}
