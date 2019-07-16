use {Day, Host};
use bloom::{ASMS, BloomFilter};
use chrono::{Date, Duration, Local};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::IpAddr;
use tiny_http::{Header, HeaderField, Response, Request};
use url::Url;

static HELLO_PIXEL: [u8; 41] = [
    // 💜
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x01, 0x00, 0xc4, 0x52, 0xc8,
    0xff, 0xff, 0xff, 0x21, 0xfe, 0x02, 0x3c, 0x33, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
    0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

fn cleanup(history: &mut Vec<Day>) {
    let today = Local::today();
    let cutoff = today - Duration::days(30);
    history.retain(|day| day.date > cutoff);
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

pub fn record(mut history: &mut Vec<Day>,
              (ip, hostname, path): (Option<IpAddr>, String, String),
              date: Date<Local>) {
    let seen_before = ip.map_or(false, |addr| {
        history.iter().any(|day| {
            day.hosts
                .get(&hostname)
                .map_or(false, |h| h.visitors.contains(&addr))
        })
    });

    if history.iter().find(|day| day.date == date).is_none() {
        cleanup(&mut history);
        let day = Day {
            date: date,
            hosts: HashMap::new(),
        };
        history.push(day);
    }
    let today = history
        .iter_mut()
        .find(|day| day.date == date)
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
}

pub fn count(request: &Request, history: &mut Vec<Day>) -> Response<Cursor<Vec<u8>>> {
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

    let view = match trackable(&request) {
        Some(x) => x,
        None => return response.with_status_code(400),
    };
    record(history, view, Local::today());

    response
}

pub fn mock(history: &mut Vec<Day>) {
    let today_date = Local::today();
    for day_count in 0..31 {
        let day = today_date - Duration::days(day_count);
        let visitors = ((day_count as f64).sin() * 100.) as i64 + 100;
        for v in 0..visitors {
            let ip = format!("0.0.0.{}", (v * day_count % 256)).parse();
            let path = format!("/hello-{}", (v / 20) % 12 + (day_count / 2) % 5);
            record(history, (ip.ok(), "hello".to_string(), path), day);
        }
    }
}
