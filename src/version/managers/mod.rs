use rustc_serialize::json::{self, Json};
use shaman::digest::Digest;

pub mod composer;

/// Hashes dependencies list in json format
///
/// Dependencies should be in the format:
/// ```json
/// {
///   "dep1": "version",
///   "dep2": "version"
/// }
/// ```
fn hash_json_deps(data: &Json, key: &str, hash: &mut Digest) {
    let deps = data.find(key);
    if let Some(&Json::Object(ref ob)) = deps {
        hash.input(key.as_bytes());
        hash.input(b"-->\0");
        // Note the BTree is sorted on its own
        for (key, val) in ob {
            hash.input(key.as_bytes());
            hash.input(val.as_string()
                .map(|x| x.as_bytes())
                .unwrap_or(b"*"));
            hash.input(b"\0");
        }
        hash.input(b"<--\0");
    }
}

fn hash_json(data: &Json, key: &str, hash: &mut Digest) {
    let data = data.find(key);
    if let Some(ref ob) = data {
        hash.input(key.as_bytes());
        let encoded = format!("{}", json::as_json(&ob));
        hash.input(encoded.as_bytes())
    }
}
