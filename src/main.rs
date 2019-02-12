use std::env;
use tiny_http::{Server, Response, HeaderField};
use url::{Url};

fn main() {
    let port = match env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(..) => 8000,
    };

    let server = Server::http(("0.0.0.0", port)).unwrap();

    let mut count = 0;

    let referer_header = HeaderField::from_bytes("Referer").unwrap();

    for request in server.incoming_requests() {
        count += 1;
        println!("req n {:?}", count);
        let referer = request.headers()
            .iter()
            .find(|header| header.field == referer_header)
            .map(|header| Url::parse(header.value.as_str()));
        let url = match referer {
            Some(r) => match r {
                Err(e) => {
                    println!("bad referer: {:?}", e);
                    continue;
                },
                Ok(d) => d,
            },
            None => {
                println!("no referrer");
                continue;
            }
        };

        println!("host: {:?}", url.host_str().unwrap());
        println!("path: {:?}", url.path());

        let response = Response::from_string("hello world");
        request.respond(response).unwrap();
    }
}
