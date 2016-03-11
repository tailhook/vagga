use std::fs::File;
use std::path::{Path, PathBuf};

use rustc_serialize::json::Json;
use shaman::digest::Digest;

use version::error::Error;
use config::builders::ComposerDepInfo;


pub fn hash(info: &ComposerDepInfo, hash: &mut Digest) -> Result<(), Error> {
    let base_path: PathBuf = {
        let path = Path::new("/work");
        if let Some(ref working_dir) = info.working_dir {
            path.join(working_dir)
        } else {
            path.to_owned()
        }
    };

    let path = base_path.join("composer.lock");
    if path.exists() { try!(
        File::open(&path).map_err(|e| Error::Io(e, path.clone()))
        .and_then(|mut f| Json::from_reader(&mut f)
            .map_err(|e| Error::Json(e, path.to_path_buf())))
        .map(|data| {
            data.find("hash").map(
                |h| h.as_string().map(|h| hash.input(h.as_bytes()))
            );
            data.find("content-hash").map(
                |h| h.as_string().map(|h| hash.input(h.as_bytes()))
            );
        })
    );}

    let path = base_path.join("composer.json");
    File::open(&path).map_err(|e| Error::Io(e, path.clone()))
    .and_then(|mut f| Json::from_reader(&mut f)
        .map_err(|e| Error::Json(e, path.to_path_buf())))
    .map(|data| {
        // just use `npm_hash_deps` here for the structure is equal
        super::hash_json_deps(&data, "require", hash);
        super::hash_json_deps(&data, "conflict", hash);
        super::hash_json_deps(&data, "replace", hash);
        super::hash_json_deps(&data, "provide", hash);
        super::hash_json_deps(&data, "suggest", hash);
        // "autoload" and "repositories" can be quite complex, just hash everything
        super::hash_json(&data, "autoload", hash);
        super::hash_json(&data, "repositories", hash);

        super::hash_json(&data, "minimum-stability", hash);
        super::hash_json(&data, "prefer-stable", hash);

        if info.dev {
            super::hash_json_deps(&data, "require-dev", hash);
            super::hash_json(&data, "autoload-dev", hash);
        }
    })
}
