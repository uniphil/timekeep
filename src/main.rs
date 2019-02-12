use std::env;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::IpAddr;
use url::{Url};
use tiny_http::{Server, Request, Response, HeaderField};
use chrono::{Date, Local, Duration};
use bloom::{ASMS, BloomFilter};


struct Host {
    paths: HashMap<String, u32>,
    visitors: BloomFilter,
    unique_visitors: u32,
    new_visitors: u32,
}

struct Day {
    date: Date<Local>,
    hosts: HashMap<String, Host>,
}

fn trackable(request: &Request) -> Option<(IpAddr, String, String)> {
    let addr = request.remote_addr().ip();
    let referer = request.headers()
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
    return Some((addr, hostname.to_string(), path.to_string()));
}

fn cleanup(history: &mut Vec<Day>) {
    let today = Local::today();
    let cutoff = today - Duration::days(30);
    history.retain(|day| day.date > cutoff);
}

fn count(request: &Request, mut history: &mut Vec<Day>) -> Response<Cursor<Vec<u8>>> {
    let (ip, hostname, path) = match trackable(&request) {
        Some(x) => x,
        None => return Response::from_string("booo"),
    };

    let today_date = Local::today();

    let seen_before = history
        .iter()
        .any(|day| day.hosts.get(&hostname)
            .map(|h| h.visitors.contains(&ip))
            .unwrap_or(false));

    let today = match history.iter_mut().find(|day| day.date == today_date) {
        Some(d) => d,
        None => {
            cleanup(&mut history);
            let day = Day{ date: today_date, hosts: HashMap::new() };
            history.push(day);
            history.iter_mut().find(|day| day.date == Local::today()).unwrap()
        }
    };

    let mut host = today.hosts.entry(hostname).or_insert(Host {
        paths: HashMap::new(),
        visitors: BloomFilter::with_rate(0.03, 10000),
        unique_visitors: 0,
        new_visitors: 0,
    });

    let new_today = host.visitors.insert(&ip);
    if new_today {
        host.unique_visitors += 1;
    }
    if !seen_before {
        host.new_visitors += 1;
    }

    *host.paths.entry(path).or_insert(0) += 1;

    Response::from_string("hello world")
}

fn index(_request: &Request, history: &Vec<Day>) -> Response<Cursor<Vec<u8>>> {
    let mut out = "about some hosts:\ndate\t\tnew folks\ttotal visitors\n".to_string();
    let mut hosts: HashMap<String, HashMap<&Date<Local>, (u32, u32)>> = HashMap::new();
    for day in history {
        let date = &day.date;
        for (host, counts) in &day.hosts {
            let h = hosts.entry(host.to_string()).or_insert(HashMap::new());
            h.insert(date, (counts.new_visitors, counts.unique_visitors));
        }
    }
    for (host, info) in hosts {
        out.push_str(&format!("\n{}\n", host));
        for (date, (new_visitors, unique_visitors)) in info {
            out.push_str(&format!("{:?}\t{:?}\t{:?}\n",
                date, new_visitors, unique_visitors));
        }
    }
    Response::from_string(out)
}

fn detail(_request: &Request, history: &Vec<Day>, hostname: &str) -> Response<Cursor<Vec<u8>>> {
    let mut out = format!("recent memories of {}:\n", hostname);
    let mut info = history
        .iter()
        .filter_map(|day| day.hosts.get(hostname).map(|h| (day.date, h)))
        .peekable();
    if info.peek().is_none() {
        out.push_str(&format!("no records :/\n"));
        return Response::from_string("nothing for u");
    }
    let mut paths = HashMap::new();
    out.push_str("date\t\timpressions\tuniques\tnew folks\n");
    for (date, h) in info {
        let mut day_visits = 0;
        for (path, count) in &h.paths {
            *paths.entry(path).or_insert(0) += count;
            day_visits += count;
        }
        out.push_str(&format!("{:?}\t{:?}\t{:?}\t{:?}\n",
            date, day_visits, h.unique_visitors, h.new_visitors));
    }
    let mut paths = paths.iter().collect::<Vec<_>>();
    paths.sort_unstable_by(|(_, &a), (_, &b)| b.cmp(&a));
    out.push_str(&format!("\nimpressions in the last 30 days by path:\n"));
    for (path, path_count) in paths {
        out.push_str(&format!("{}\t{}\n", path_count, path));
    }
    Response::from_string(out)
}

fn main() {
    let port = match env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(..) => 8000,
    };
    let server = Server::http(("0.0.0.0", port)).unwrap();
    let mut history: Vec<Day> = Vec::new();

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/count.gif" => count(&request, &mut history),
            "/" => index(&request, &history),
            hostname => detail(&request, &history, hostname.get(1..).unwrap()),
        };
        request.respond(response).unwrap();
    }
}
