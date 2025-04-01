use anyhow::{Result, anyhow};
use regex::Regex;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    sync::OnceLock,
};

static LOG_REGEX: OnceLock<Regex> = OnceLock::new();

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: String,
    datetime: String,
    method: String,
    url: String,
    protocol: String,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

fn main() -> Result<()> {
    let path = env::current_dir()?
        .join("grammar")
        .join("assets")
        .join("nginx_log.txt");

    let mut nginx_log_vec = Vec::new();

    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    for line in buffered.lines() {
        let line = line?;
        let nginx_log = parse_nginx_log(&line)?;
        nginx_log_vec.push(nginx_log);
    }

    println!("{:#?}", nginx_log_vec[5]);

    Ok(())
}

fn parse_nginx_log(s: &str) -> Result<NginxLog> {
    let re = LOG_REGEX.get_or_init(|| {
        Regex::new(r#"^(?<ip>\S+)\s+\S+\s+\S+\s+\[(?<date>[^\]]+)\]\s+"(?<method>\S+)\s+(?<url>\S+)\s+(?<proto>[^"]+)"\s+(?<status>\d+)\s+(?<bytes>\d+)\s+"(?<referer>[^"]+)"\s+"(?<ua>[^"]+)"$"#,).unwrap()
    });
    let cap = re.captures(s).ok_or(anyhow!("parse error"))?;

    let addr = cap
        .name("ip")
        .map(|ip| ip.as_str().to_string())
        .ok_or(anyhow!("parse ip error"))?;

    let datetime = cap
        .name("date")
        .map(|date| date.as_str().to_string())
        .ok_or(anyhow!("parse date error"))?;

    let method = cap
        .name("method")
        .map(|method| method.as_str().to_string())
        .ok_or(anyhow!("parse method error"))?;

    let url = cap
        .name("url")
        .map(|url| url.as_str().to_string())
        .ok_or(anyhow!("parse url error"))?;

    let protocol = cap
        .name("proto")
        .map(|proto| proto.as_str().to_string())
        .ok_or(anyhow!("parse proto error"))?;

    let status = cap
        .name("status")
        .map(|status| status.as_str().parse())
        .ok_or(anyhow!("parse status error"))??;

    let body_bytes = cap
        .name("bytes")
        .map(|bytes| bytes.as_str().parse())
        .ok_or(anyhow!("parse bytes error"))??;

    let referer = cap
        .name("referer")
        .map(|referer| referer.as_str().to_string())
        .ok_or(anyhow!("parse referer error"))?;

    let user_agent = cap
        .name("ua")
        .map(|ua| ua.as_str().to_string())
        .ok_or(anyhow!("parse ua error"))?;

    Ok(NginxLog {
        addr,
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
