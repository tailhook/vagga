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

fn hash_json_recursive(data: &Json, hash: &mut Digest) {
    match data {
        &Json::Object(ref ob) => {
            for (k, v) in ob.iter() {
                hash.input(k.as_bytes());
                hash_json_recursive(v, hash);
            }
        }
        &Json::Array(ref ar) => {
            for i in ar.iter() {
                hash_json_recursive(i, hash);
            }
        }
        &Json::String(ref val) => hash.input(val.as_bytes()),
        &Json::U64(val) => hash.input(format!("{}", val).as_bytes()),
        &Json::I64(val) => hash.input(format!("{}", val).as_bytes()),
        &Json::F64(val) => hash.input(format!("{}", val).as_bytes()),
        &Json::Boolean(val) => {
            if val { hash.input(b"true") }
            else { hash.input(b"false") }
        }
        _ => {}
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
