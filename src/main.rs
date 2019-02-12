use std::env;
use std::collections::HashMap;
use url::{Url};
use tiny_http::{Server, Response, HeaderField};

fn main() {
    let port = match env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(..) => 8000,
    };
    let referer_header = HeaderField::from_bytes("Referer").unwrap();

    let server = Server::http(("0.0.0.0", port)).unwrap();

    let mut by_parts: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for request in server.incoming_requests() {
        let referer = request.headers()
            .iter()
            .find(|header| header.field == referer_header)
            .map(|header| Url::parse(header.value.as_str()));
        let url = match referer {
            Some(r) => match r {
                Err(_) => continue,
                Ok(d) => d,
            },
            None => continue,
        };
        let host = match url.host_str() {
            Some(h) => h,
            None => continue,
        };
        let path = url.path();

        let counter = by_parts
            .entry(host.to_string()).or_insert(HashMap::new())
            .entry(path.to_string()).or_insert(0);
        *counter += 1;

        println!("{} – {} {}", counter, host, path);

        let response = Response::from_string("hello world");
        request.respond(response).unwrap();
    }
}
