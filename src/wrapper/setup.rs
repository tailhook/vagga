use std::os::{getenv};
use std::io::BufferedReader;
use std::io::fs::File;
use std::collections::TreeMap;

use config::Container;
use super::run::DEFAULT_PATH;


pub fn get_environment(container: &Container)
    -> Result<TreeMap<String, String>, String>
{
    let mut result = TreeMap::new();
    result.insert("TERM".to_string(),
                  getenv("TERM").unwrap_or("dumb".to_string()));
    result.insert("PATH".to_string(),
                  DEFAULT_PATH.to_string());
    if let Some(ref filename) = container.environ_file {
        let mut f = BufferedReader::new(try!(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e))));
        for line_read in f.lines() {
            let line = try!(line_read
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e)));
            let mut pair = line.as_slice().splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = try!(pair.next()
                .ok_or(format!("Error reading environment file {}: bad format",
                    filename.display())));
            if value.len() > 0 && value.starts_with("\"") {
                value = value[ 1 .. value.len()-2 ];
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    for (ref k, ref v) in container.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    return Ok(result);
}
