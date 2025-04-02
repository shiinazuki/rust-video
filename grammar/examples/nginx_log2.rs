use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    time::Instant,
};

use chrono::{DateTime, Utc};
use winnow::{
    Parser,
    ascii::{digit1, space0},
    combinator::{alt, delimited, separated},
    token::take_until,
};

#[derive(Debug, PartialEq, Eq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "CONNECT" => Ok(HttpMethod::Connect),
            "TRACE" => Ok(HttpMethod::Trace),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(anyhow::anyhow!("Invalid HTTP method")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum HttpProto {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl FromStr for HttpProto {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(HttpProto::HTTP1_0),
            "HTTP/1.1" => Ok(HttpProto::HTTP1_1),
            "HTTP/2.0" => Ok(HttpProto::HTTP2_0),
            "HTTP/3.0" => Ok(HttpProto::HTTP3_0),
            _ => Err(anyhow::anyhow!("Invalid HTTP proto")),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: IpAddr,
    datetime: DateTime<Utc>,
    method: HttpMethod,
    url: String,
    protocol: HttpProto,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

impl NginxLog {
    fn new() -> Self {
        NginxLog {
            addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            datetime: Utc::now(),
            method: HttpMethod::Get,
            url: "".to_string(),
            protocol: HttpProto::HTTP1_0,
            status: 200,
            body_bytes: 0,
            referer: "".to_string(),
            user_agent: "".to_string(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let path = env::current_dir()?
        .join("grammar")
        .join("assets")
        .join("nginx_log.txt");

    let mut nginx_log_vec = Vec::new();

    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    for line in buffered.lines() {
        let line = line?;
        let nginx_log = parse_nginx_log(&line).unwrap_or(NginxLog::new());
        // let nginx_log =
        // parse_nginx_log(&line).map_err(|e| anyhow::anyhow!("Failed to parse log: {:?}", e))?;
        nginx_log_vec.push(nginx_log);
    }

    let duration = start.elapsed();
    println!("{}", duration.as_millis());

    println!("{:#?}", nginx_log_vec[5]);
    Ok(())
}

fn parse_nginx_log(s: &str) -> winnow::Result<NginxLog> {
    let input = &mut (&*s);
    let ip = parse_ip(input)?;
    parse_ignored(input)?;
    parse_ignored(input)?;
    let datetime = parse_datetime(input)?;
    let (method, url, protocol) = parse_http(input)?;
    let status = parse_status(input)?;
    let body_bytes = parse_body_bytes(input)?;
    let referer = parse_quoted_string(input)?;
    let user_agent = parse_quoted_string(input)?;

    Ok(NginxLog {
        addr: ip,
        datetime,
        method,
        url,
        protocol,
        status,
        body_bytes,
        referer,
        user_agent,
    })
}

fn parse_ip(s: &mut &str) -> winnow::Result<IpAddr> {
    let ret: Vec<u8> = separated(4, digit1.parse_to::<u8>(), '.').parse_next(s)?;
    space0(s)?;
    Ok(IpAddr::V4(Ipv4Addr::new(ret[0], ret[1], ret[2], ret[3])))
}

fn parse_ignored(s: &mut &str) -> winnow::Result<()> {
    "- ".parse_next(s)?;
    Ok(())
}

fn parse_datetime(s: &mut &str) -> winnow::Result<DateTime<Utc>> {
    let ret = delimited('[', take_until(1.., ']'), ']').parse_next(s)?;
    space0(s)?;
    let datetime = DateTime::parse_from_str(ret, "%d/%b/%Y:%H:%M:%S %z")
        .unwrap()
        .to_utc();

    Ok(datetime)
}

fn parse_http(s: &mut &str) -> winnow::Result<(HttpMethod, String, HttpProto)> {
    let parser = (parse_method, parse_url, parse_protocol);
    let ret = delimited('"', parser, '"').parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_method(s: &mut &str) -> winnow::Result<HttpMethod> {
    let ret = alt((
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "TRACE", "PATCH",
    ))
    .parse_to()
    .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_url(s: &mut &str) -> winnow::Result<String> {
    let ret = take_until(1.., ' ').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

fn parse_protocol(s: &mut &str) -> winnow::Result<HttpProto> {
    let ret = alt(("HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"))
        .parse_to()
        .parse_next(s)?;
    space0(s)?;

    Ok(ret)
}

fn parse_status(s: &mut &str) -> winnow::Result<u16> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_body_bytes(s: &mut &str) -> winnow::Result<u64> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_quoted_string(s: &mut &str) -> winnow::Result<String> {
    let ret = delimited('"', take_until(1.., '"'), '"').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parse_ip_should_work() -> anyhow::Result<()> {
        let mut s = "1.1.1.1";
        let ip = parse_ip(&mut s).unwrap();

        assert_eq!(s, "");
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));

        Ok(())
    }

    #[test]
    fn parse_datetime_should_work() -> anyhow::Result<()> {
        let mut s = "[17/May/2015:08:05:32 +0000]";
        let dt = parse_datetime(&mut s).unwrap();

        assert_eq!(s, "");

        assert_eq!(dt, Utc.with_ymd_and_hms(2015, 5, 17, 8, 5, 32).unwrap());

        Ok(())
    }

    #[test]
    fn parse_http_should_work() -> anyhow::Result<()> {
        let mut s = "\"GET /downloads/product_1 HTTP/1.1\"";
        let (method, url, protocol) = parse_http(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(method, HttpMethod::Get);
        assert_eq!(url, "/downloads/product_1");
        assert_eq!(protocol, HttpProto::HTTP1_1);
        Ok(())
    }
}
