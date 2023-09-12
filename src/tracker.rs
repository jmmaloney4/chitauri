use url::Url;

enum Protocol {
    HTTP,
    UDP,
}

trait Tracker {}

struct HTTPTracker {
    url: Url,
}

impl Tracker for HTTPTracker {}
