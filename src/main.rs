extern crate bloom;
extern crate chrono;
extern crate tiny_http;
extern crate url;

use bloom::{BloomFilter, ASMS};
use chrono::{Date, DateTime, Duration, Local};
use std::collections::HashMap;
use std::env;
use std::io::Cursor;
use std::net::IpAddr;
use tiny_http::{Header, HeaderField, Request, Response, Server};
use url::Url;

static HELLO_PIXEL: [u8; 41] = [
    // 💜
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x01, 0x00, 0xc4, 0x52, 0xc8,
    0xff, 0xff, 0xff, 0x21, 0xfe, 0x02, 0x3c, 0x33, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
    0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

struct Host {
    paths: HashMap<String, u32>,
    visitors: BloomFilter,
    unique_visitors: u32,
    new_visitors: u32,
    dnt_impressions: u32,
}

struct Day {
    date: Date<Local>,
    hosts: HashMap<String, Host>,
}

fn trackable(request: &Request) -> Option<(Option<IpAddr>, String, String)> {
    let dnt = request
        .headers()
        .iter()
        .find(|h| h.field == HeaderField::from_bytes("DNT").unwrap())
        .map_or(false, |h| h.value.as_str() == "1");
    let addr = if dnt {
        None
    } else {
        Some(
            request
                .headers()
                .iter()
                .find(|header| header.field == HeaderField::from_bytes("X-Forwarded-For").unwrap())
                .and_then(|header| header.value.as_str().parse().ok())
                .unwrap_or_else(|| request.remote_addr().ip()),
        )
    };
    let referer = request
        .headers()
        .iter()
        .find(|header| header.field == HeaderField::from_bytes("Referer").unwrap())
        .map(|header| Url::parse(header.value.as_str()));
    let url = match referer {
        Some(r) => match r {
            Err(_) => return None,
            Ok(d) => d,
        },
        None => return None,
    };
    let hostname = match url.host_str() {
        Some(h) => h,
        None => return None,
    };
    let path = url.path();
    Some((addr, hostname.to_string(), path.to_string()))
}

fn cleanup(history: &mut Vec<Day>) {
    let today = Local::today();
    let cutoff = today - Duration::days(30);
    history.retain(|day| day.date > cutoff);
}

fn count(request: &Request, mut history: &mut Vec<Day>) -> Response<Cursor<Vec<u8>>> {
    let response = Response::from_data(HELLO_PIXEL.to_vec())
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"image/gif"[..]).unwrap())
        .with_header(
            Header::from_bytes(
                &b"Cache-Control"[..],
                &b"no-store, no-cache, must-revalidate, max-age=0"[..],
            )
            .unwrap(),
        )
        .with_header(Header::from_bytes(&b"Pragma"[..], &b"no-cache"[..]).unwrap());
    let (ip, hostname, path) = match trackable(&request) {
        Some(x) => x,
        None => return response.with_status_code(400),
    };

    let today_date = Local::today();

    let seen_before = ip.map_or(false, |addr| {
        history.iter().any(|day| {
            day.hosts
                .get(&hostname)
                .map_or(false, |h| h.visitors.contains(&addr))
        })
    });

    if history.iter().find(|day| day.date == today_date).is_none() {
        cleanup(&mut history);
        let day = Day {
            date: today_date,
            hosts: HashMap::new(),
        };
        history.push(day);
    }
    let today = history
        .iter_mut()
        .find(|day| day.date == Local::today())
        .unwrap();

    let host = today.hosts.entry(hostname).or_insert(Host {
        paths: HashMap::new(),
        visitors: BloomFilter::with_rate(0.03, 10000),
        unique_visitors: 0,
        new_visitors: 0,
        dnt_impressions: 0,
    });

    if let Some(addr) = ip {
        let new_today = host.visitors.insert(&addr);
        if new_today {
            host.unique_visitors += 1;
        }
        if !seen_before {
            host.new_visitors += 1;
        }
    } else {
        host.dnt_impressions += 1;
    }

    *host.paths.entry(path).or_insert(0) += 1;

    response
}

fn index(
    _request: &Request,
    history: &[Day],
    launch: &DateTime<Local>,
) -> Response<Cursor<Vec<u8>>> {
    type Detail<'a> = HashMap<&'a Date<Local>, (u32, u32, u32)>;
    let mut out =
        "<!doctype html><pre>about some hosts:\ndate\tnew folks\tdaily visits\tdnt impressions\n"
            .to_string();
    let mut hosts: HashMap<String, Detail> = HashMap::new();
    for day in history {
        let date = &day.date;
        for (host, counts) in &day.hosts {
            let h = hosts.entry(host.to_string()).or_insert_with(HashMap::new);
            h.insert(
                date,
                (
                    counts.new_visitors,
                    counts.unique_visitors,
                    counts.dnt_impressions,
                ),
            );
        }
    }
    let mut hosts = hosts.iter().collect::<Vec<_>>();
    hosts.sort_by_key(|&(h, _)| h);
    for (host, info) in hosts {
        let mut total_new = 0;
        let mut total_unique = 0;
        let mut total_dnt = 0;
        let mut timeline = "".to_string();

        let mut info = info.iter().collect::<Vec<_>>();
        info.sort_by_key(|&(date, _)| date);
        info.reverse();

        for (date, (new_visitors, unique_visitors, dnt_impressions)) in info {
            total_new += new_visitors;
            total_unique += unique_visitors;
            total_dnt += dnt_impressions;
            timeline.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                date.format("%F"),
                new_visitors,
                unique_visitors,
                dnt_impressions
            ));
        }
        out.push_str(&format!(
            "\n<a href=\"/{0}\">{}</a>\t{}\t{}\t{}\n",
            host, total_new, total_unique, total_dnt
        ));
        out.push_str(&timeline);
    }
    out.push_str(&format!("\n\nlast restart: {}", launch));
    out.push_str("</pre>");
    Response::from_string(out)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap())
}

fn detail(_request: &Request, history: &[Day], hostname: &str) -> Response<Cursor<Vec<u8>>> {
    let mut out = format!("<!doctype html><pre>recent memories of <a href=\"https://{0}/\" target=\"_blank\">{0} ⎘</a>:\n", hostname);
    let mut info = history
        .iter()
        .filter_map(|day| day.hosts.get(hostname).map(|h| (day.date, h)))
        .peekable();
    if info.peek().is_none() {
        out.push_str("no records :/\n");
        return Response::from_string(out)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
    }
    let mut paths = HashMap::new();
    let mut info = info.collect::<Vec<_>>();
    info.sort_by_key(|&(d, _)| d);
    info.reverse();
    out.push_str("date\timpressions\tdnt\tvisitors\tnew folks\n");
    for (date, h) in info {
        let mut day_visits = 0;
        for (path, count) in &h.paths {
            *paths.entry(path).or_insert(0) += count;
            day_visits += count;
        }
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            date.format("%F"),
            day_visits,
            h.dnt_impressions,
            h.unique_visitors,
            h.new_visitors
        ));
    }
    let mut paths = paths.iter().collect::<Vec<_>>();
    paths.sort_by_key(|&(path, count)| (count, path));
    paths.reverse();
    out.push_str("\nimpressions in the last 30 days by path:\n");
    for (path, path_count) in paths {
        out.push_str(&format!(
            "{}\t<a href=\"https://{2}{1}\" target=\"_blank\">{1} ⎘</a>\n",
            path_count, path, hostname
        ));
    }
    out.push_str("</pre>");
    Response::from_string(out)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap())
}

fn dnt_policy(dnt_compliant: bool) -> Response<Cursor<Vec<u8>>> {
    if dnt_compliant {
        let policy = include_str!("dnt-policy-1.0.txt");
        Response::from_string(policy)
    } else {
        Response::from_string("".to_string()).with_status_code(404)
    }
}

fn main() {
    let port = match env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(..) => 8000,
    };
    let dnt_compliant = env::var("DNT_COMPLIANT").ok().map_or(false, |c| c == "1");
    let server = Server::http(("0.0.0.0", port)).unwrap();
    let mut history: Vec<Day> = Vec::new();
    let launch = Local::now();

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/count.gif" => count(&request, &mut history),
            "/" => index(&request, &history, &launch),
            "/.well-known/dnt-policy.txt" => dnt_policy(dnt_compliant),
            hostname => detail(&request, &history, hostname.get(1..).unwrap()),
        };
        request.respond(response).unwrap();
    }
    println!("got to end of requests apparently. bye!");
}
