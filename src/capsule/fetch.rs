use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::net::ToSocketAddrs;

use sha2::{Digest, Sha256};
use url::Url;
use indicatif::{MultiProgress, ProgressBar};
use tk_easyloop;
use digest::hex;


#[derive(Debug)]
pub struct FetchTask {
    pub url: Url,
    pub sha256: Option<String>,
    pub destination: PathBuf,
}

impl FetchTask {
    pub fn cache(url_str: &str, sha256: &Option<String>) -> FetchTask {
        let url = url_str.parse::<Url>().expect("valid url");

        let hash = match *sha256 {
            Some(ref sha256) => sha256[..8].to_string(),
            None => {
                let mut hash = Sha256::new();
                hash.input(url_str.as_bytes());
                format!("{:.8x}", hex(&hash))
            },
        };
        let dest = {
            let path = Path::new(url.path());
            let name = match path.file_name().and_then(|x| x.to_str()) {
                Some(name) => name,
                None => "file.bin",
            };
            let name = hash[..8].to_string() + "-" + name;
            let dir = Path::new("/vagga/cache/downloads");
            dir.join(&name)
        };

        FetchTask {
            // TODO(tailhook) don't crash
            url: url,
            sha256: sha256.clone(),
            destination: dest,
        }
    }
}


pub fn fetch_many(tasks: Vec<FetchTask>) {
    let all = Arc::new(MultiProgress::new());
    tk_easyloop::run(|| {
        // we can't afford threads so let's just resolve synchronously
        for t in tasks {
            let ip = match (t.url.host_str(), t.url.port_or_known_default()) {
                (Some(h), Some(p)) => match (h, p).to_socket_addrs() {
                    Ok(mut iter) => match iter.next() {
                        Some(addr) => addr,
                        None => {
                            error!("Error resolving host {:?}: \
                                    zero addresses returned", t.url);
                            continue;
                        }
                    },
                    Err(e) => {
                        error!("Error resolving host {:?}: {}",
                            t.url, e);
                        continue;
                    }
                },
                _ => {
                    error!("Invalid url {:?}", t.url);
                    continue;
                }
            };
            let prog = all.add(ProgressBar::new(80));
            prog.tick();
            println!("Download {} from {}", t.url, ip);
        }
        Ok(())
    }).map_err(|e: Box<::std::error::Error>|
        error!("Error prefetching: {}", e)).ok();
}
