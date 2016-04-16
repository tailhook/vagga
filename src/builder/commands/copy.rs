use std::io::ErrorKind;
use std::fs::{symlink_metadata};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{PermissionsExt, MetadataExt};

use libc::{uid_t, gid_t};
use regex::Regex;
use scan_dir::{ScanDir};

use file_util::{create_dir_mode, shallow_copy};
use path_util::ToRelative;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(RustcDecodable, Debug)]
pub struct Copy {
    pub source: PathBuf,
    pub path: PathBuf,
    pub owner_uid: Option<uid_t>,
    pub owner_gid: Option<gid_t>,
    pub ignore_regex: String,
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
                                let path = fpath.rel_to(src).unwrap();
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
        let ref src = self.source;
        let dest = Path::new("/vagga/root").join(self.path.rel());
        let typ = try!(symlink_metadata(src)
            .map_err(|e| StepError::Write(src.into(), e)));
        if typ.is_dir() {
            try!(create_dir_mode(&dest, typ.permissions().mode())
                .map_err(|e| StepError::Write(dest.clone(), e)));
            let re = try!(Regex::new(&self.ignore_regex)
                .map_err(|e| StepError::Regex(Box::new(e))));
            try!(ScanDir::all().walk(src, |iter| {
                for (entry, _) in iter {
                    let fpath = entry.path();
                    // We know that directory is inside
                    // the source
                    let path = fpath.rel_to(src).unwrap();
                    // We know that it's decodable
                    let strpath = path.to_str().unwrap();
                    if re.is_match(strpath) {
                        continue;
                    }
                    let fdest = dest.join(path);
                    try!(shallow_copy(&fpath, &fdest,
                            self.owner_uid, self.owner_gid)
                        .map_err(|e| StepError::Write(fdest, e)));
                }
                Ok(())
            }).map_err(StepError::ScanDir).and_then(|x| x));
        } else {
            try!(shallow_copy(&self.source, &dest,
                              self.owner_uid, self.owner_gid)
                 .map_err(|e| StepError::Write(dest.clone(), e)));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
