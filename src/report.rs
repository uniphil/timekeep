use std::collections::HashMap;
use std::io::Cursor;

use Day;
use chrono::{Date, DateTime, Local};
use tiny_http::{Header, Request, Response,};

pub fn index(
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


pub fn detail(_request: &Request, history: &[Day], hostname: &str) -> Response<Cursor<Vec<u8>>> {
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


pub fn dnt_policy(dnt_compliant: bool) -> Response<Cursor<Vec<u8>>> {
    if dnt_compliant {
        let policy = include_str!("dnt-policy-1.0.txt");
        Response::from_string(policy)
    } else {
        Response::from_string("".to_string()).with_status_code(404)
    }
}
