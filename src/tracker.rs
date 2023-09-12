use url::Url;

enum Protocol {
    HTTP,
    UDP,
}

trait Tracker {
    fn get_peers(&self) -> Vec<String>;
}

struct HTTPTracker {
    url: Url,
    client: reqwest::Client,
}

impl Tracker for HTTPTracker {
    fn get_peers(&self) -> Vec<String> {
        Vec::new()
    }
}
