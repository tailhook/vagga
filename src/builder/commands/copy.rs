use std::io::ErrorKind;
use std::fs::{symlink_metadata};
use std::path::PathBuf;
use std::os::unix::fs::{PermissionsExt, MetadataExt};

use libc::{uid_t, gid_t};
use quire::validate as V;
use regex::Regex;
use scan_dir::{ScanDir};

use container::root::temporary_change_root;
use file_util::{create_dir_mode, shallow_copy};
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use quick_error::ResultExt;


#[derive(RustcDecodable, Debug)]
pub struct Copy {
    pub source: PathBuf,
    pub path: PathBuf,
    pub owner_uid: Option<uid_t>,
    pub owner_gid: Option<gid_t>,
    pub ignore_regex: String,
}

impl Copy {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("source", V::Scalar::new())
        .member("path", V::Directory::new().is_absolute(true))
        .member("ignore_regex", V::Scalar::new().default(
            r#"(^|/)\.(git|hg|svn|vagga)($|/)|~$|\.bak$|\.orig$|^#.*#$"#))
        .member("owner_uid", V::Numeric::new().min(0).optional())
        .member("owner_gid", V::Numeric::new().min(0).optional())
    }
}

impl BuildStep for Copy {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let ref src = self.source;
        if src.starts_with("/work") {
            match symlink_metadata(src) {
                Ok(ref meta) if meta.file_type().is_dir() => {
                    let re = try!(Regex::new(&self.ignore_regex)
                        .map_err(|e| VersionError::Regex(Box::new(e))));
                    try!(ScanDir::all().walk(src, |iter| {
                        let mut all_entries = iter.filter_map(|(e, _)|
                        {
                            let fpath = e.path();
                            let strpath = {
                                // We know that directory is inside
                                // the source
                                let path = fpath.strip_prefix(src).unwrap();
                                // We know that it's decodable
                                let strpath = path.to_str().unwrap();
                                if !re.is_match(strpath) {
                                    Some(strpath.to_string())
                                } else {
                                    None
                                }
                            };
                            strpath.map(|x| (fpath, x))
                        }).collect::<Vec<_>>();
                        all_entries.sort();
                        for (fpath, name) in all_entries {
                            hash.field("filename", name);
                            try!(hash.file(&fpath,
                                self.owner_uid, self.owner_gid)
                                .map_err(|e| VersionError::Io(e, fpath)));
                        }
                        Ok(())
                    }).map_err(VersionError::ScanDir).and_then(|x| x));
                }
                Ok(_) => {
                    try!(hash.file(src, self.owner_uid, self.owner_gid)
                        .map_err(|e| VersionError::Io(e, src.into())));
                }
                Err(ref e) if e.kind() == ErrorKind::NotFound => {
                    return Err(VersionError::New);
                }
                Err(e) => {
                    return Err(VersionError::Io(e, src.into()));
                }

            }
        } else {
            // We don't version the files outside of the /work because
            // we believe they are result of the commands run above
            //
            // And we need already built container to version the files
            // inside the container which is ugly
        }
        Ok(())
    }
    fn build(&self, _guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if !build {
            return Ok(());
        }
        temporary_change_root("/vagga/root", || {
            let ref src = self.source;
            let dest = &self.path;
            let typ = try!(symlink_metadata(src)
                .map_err(|e| StepError::Read(src.into(), e)));
            if typ.is_dir() {
                try!(create_dir_mode(dest, typ.permissions().mode())
                    .map_err(|e| StepError::Write(dest.clone(), e)));
                let re = try!(Regex::new(&self.ignore_regex)
                    .map_err(|e| StepError::Regex(Box::new(e))));
                try!(ScanDir::all().walk(src, |iter| {
                    for (entry, _) in iter {
                        let fpath = entry.path();
                        // We know that directory is inside
                        // the source
                        let path = fpath.strip_prefix(src).unwrap();
                        // We know that it's decodable
                        let strpath = path.to_str().unwrap();
                        if re.is_match(strpath) {
                            continue;
                        }
                        let fdest = dest.join(path);
                        try!(shallow_copy(&fpath, &fdest,
                                self.owner_uid, self.owner_gid)
                            .context((&fpath, &fdest)));
                    }
                    Ok(())
                }).map_err(StepError::ScanDir).and_then(|x| x));
            } else {
                try!(shallow_copy(&self.source, dest,
                                  self.owner_uid, self.owner_gid)
                    .context((&self.source, dest)));
            }
            Ok(())
        })
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
