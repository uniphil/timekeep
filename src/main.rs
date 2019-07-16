extern crate bloom;
extern crate chrono;
extern crate tiny_http;
extern crate url;

mod count;
mod report;

use count::count;
use bloom::BloomFilter;
use chrono::{Date, Local};
use report::{detail, dnt_policy, index};
use std::collections::HashMap;
use std::env;
use std::io::ErrorKind as IoErrorKind;
use tiny_http::Server;

struct Host {
    paths: HashMap<String, u32>,
    visitors: BloomFilter,
    unique_visitors: u32,
    new_visitors: u32,
    dnt_impressions: u32,
}

pub struct Day {
    date: Date<Local>,
    hosts: HashMap<String, Host>,
}


fn main() {
    let port = env::var("PORT")
        .map(|p| p.parse::<u16>().unwrap())
        .unwrap_or(8000);
    let dnt_compliant = env::var("DNT_COMPLIANT")
        .ok()
        .map_or(false, |c| c == "1");

    let mut server = Server::http(("0.0.0.0", port)).unwrap();
    let mut history: Vec<Day> = Vec::new();
    let launch = Local::now();

    loop {
        let request = match server.recv() {
            Ok(r) => r,
            Err(e) => {
                if e.kind() == IoErrorKind::ConnectionAborted {
                    println!("connection aborted: {:?}", e);
                    // apparently stuff just stops working after the abort
                    // so... try recreating the server???
                    server = Server::http(("0.0.0.0", port)).unwrap();
                    continue
                }
                println!("error: {:?}", e);
                break
            }
        };
        let response = match request.url() {
            "/count.gif" => count(&request, &mut history),
            "/" => index(&request, &history, &launch),
            "/.well-known/dnt-policy.txt" => dnt_policy(dnt_compliant),
            hostname => detail(&request, &history, hostname.get(1..).unwrap()),
        };
        if let Err(e) = request.respond(response) {
            println!("response errored: {:?}", e);
        }
    }
    println!("hit a snag apparently. bye!");
}
