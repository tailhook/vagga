use std::io::{Read, BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

use regex::Regex;
use shaman::digest::Digest;

use config::builders::GemBundleInfo;
use version::error::Error;

pub fn hash(info: &GemBundleInfo, hash: &mut Digest)
    -> Result<(), Error>
{
    let path = Path::new("/work").join(&info.gemfile);

    let gemlock = try!(path.parent()
        .map(|dir| dir.join("Gemfile.lock"))
        .ok_or("Gemfile should be under /work".to_owned()));
    if gemlock.exists() {
        try!(hash_lock_file(&gemlock, hash));
    }

    let f = try!(File::open(&path).map_err(|e| Error::Io(e, path.clone())));
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = try!(line.map_err(|e| Error::Io(e, path.clone())));
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue
        }
        hash.input(line.as_bytes());
    }

    Ok(())
}

fn hash_lock_file(path: &Path, hash: &mut Digest) -> Result<(), Error> {
    let f = try!(File::open(path).map_err(|e| Error::Io(e, path.to_path_buf())));
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = try!(line.map_err(|e| Error::Io(e, path.to_path_buf())));
        let line = line.trim();

        if line.is_empty() { continue }
        hash.input(line.as_bytes());
    }

    Ok(())
}
