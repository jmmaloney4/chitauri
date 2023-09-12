use url::Url;

use crate::torrent::{InfoHash, PeerId};

enum Protocol {
    HTTP,
    UDP,
}

trait Tracker {
    fn get_peers(&self, info_hash: InfoHash, peer_id: PeerId) -> Vec<String>;
}

struct HTTPTracker {
    url: Url,
}

impl HTTPTracker {
    fn new(url: Url) -> Self {
        Self { url }
    }
}

impl Tracker for HTTPTracker {
    fn get_peers(&self, info_hash: InfoHash, peer_id: PeerId) -> Vec<String> {
        let mut url = self.url.clone();
        url.query_pairs_mut()
            .append_pair("info_hash", &info_hash.to_hex_string())
            .append_pair("peer_id", &peer_id.to_string());
        // self.client.get(self.url)
        Vec::new()
    }
}
