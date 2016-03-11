use std::fs::File;
use std::path::{Path, PathBuf};

use rustc_serialize::json::Json;
use shaman::digest::Digest;

use version::error::Error;
use config::builders::ComposerDepInfo;

const LOCKFILE_RELEVANT_KEYS: &'static [&'static str] = &[
    "name",
    "version",
    "source",
    "dist",
    "extra",
    "autoload",
];


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
    if path.exists() {
        try!(hash_lock_file(&path, hash));
    }

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

fn hash_lock_file(path: &Path, hash: &mut Digest) -> Result<(), Error> {
    File::open(&path).map_err(|e| Error::Io(e, path.to_path_buf()))
    .and_then(|mut f| Json::from_reader(&mut f)
        .map_err(|e| Error::Json(e, path.to_path_buf())))
    .and_then(|data| {
        let packages = try!(data.find("packages")
            .ok_or("Missing 'packages' property from composer.lock".to_owned()));
        let packages = try!(packages.as_array()
            .ok_or("'packages' property is not an array".to_owned()));
        for package in packages.iter() {
            hash.input(b"-->\0");
            for key in LOCKFILE_RELEVANT_KEYS.iter() {
                hash.input(key.as_bytes());
                if let Some(jsn) = package.find(key) {
                    super::hash_json_recursive(jsn, hash)
                }
            }
            super::hash_json_deps(&package, "require", hash);
            super::hash_json_deps(&package, "require-dev", hash);
            hash.input(b"<--\0");
        }
        Ok(())
    })
}
