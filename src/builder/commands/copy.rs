use std::io::ErrorKind;
use std::fs::{symlink_metadata};
use std::path::PathBuf;
use std::os::unix::fs::{PermissionsExt, MetadataExt};
use std::collections::BTreeSet;

use libc::{uid_t, gid_t};
use quire::validate as V;
use regex::{Regex, Error as RegexError};
use scan_dir::ScanDir;

use container::root::temporary_change_root;
use file_util::{create_dir_mode, shallow_copy};
use path_util::IterSelfAndParents;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use quick_error::ResultExt;


#[derive(RustcDecodable, Debug)]
pub struct Copy {
    pub source: PathBuf,
    pub path: PathBuf,
    pub owner_uid: Option<uid_t>,
    pub owner_gid: Option<gid_t>,
    pub ignore_regex: String,
    pub include_regex: Option<String>,
}

impl Copy {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("source", V::Scalar::new())
        .member("path", V::Directory::new().is_absolute(true))
        .member("ignore_regex", V::Scalar::new().default(
            r#"(^|/)\.(git|hg|svn|vagga)($|/)|~$|\.bak$|\.orig$|^#.*#$"#))
        .member("include_regex", V::Scalar::new().optional())
        .member("owner_uid", V::Numeric::new().min(0).optional())
        .member("owner_gid", V::Numeric::new().min(0).optional())
    }

    fn get_filter(&self) -> Result<Filter, RegexError> {
        Ok(Filter {
            ignore_re: Some(try!(Regex::new(&self.ignore_regex))),
            include_re: match self.include_regex {
                Some(ref include_regex) => {
                    Some(try!(Regex::new(include_regex)))
                },
                None => None,
            },
        })
    }
}

struct Filter {
    ignore_re: Option<Regex>,
    include_re: Option<Regex>,
}

impl Filter {
    fn is_match(&self, s: &str) -> bool
    {
        self.ignore_re.as_ref().map_or(true, |r| !r.is_match(s))
            && self.include_re.as_ref().map_or(true, |r| r.is_match(s))
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
                    let filter = try!(self.get_filter());
                    try!(ScanDir::all().walk(src, |iter| {
                        let mut all_paths = BTreeSet::new();
                        for (entry, _) in iter {
                            let fpath = entry.path();
                            // We know that directory is inside
                            // the source
                            let path = fpath.strip_prefix(src).unwrap();
                            // We know that path is decodable
                            let strpath = path.to_str().unwrap();
                            if filter.is_match(strpath) {
                                for parent in path.iter_self_and_parents() {
                                    if all_paths.contains(parent) {
                                        break;
                                    }
                                    all_paths.insert(PathBuf::from(parent));
                                }
                            }
                        }
                        for cur_path in all_paths {
                            hash.field("filename", cur_path.to_str().unwrap());
                            try!(hash.file(&src.join(&cur_path),
                                     self.owner_uid, self.owner_gid)
                                 .map_err(|e| VersionError::Io(e, PathBuf::from(cur_path))));
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
                let filter = try!(self.get_filter());
                let mut processed_paths = BTreeSet::new();
                try!(ScanDir::all().walk(src, |iter| {
                    for (entry, _) in iter {
                        let fpath = entry.path();
                        // We know that directory is inside
                        // the source
                        let path = fpath.strip_prefix(src).unwrap();
                        // We know that it's decodable
                        let strpath = path.to_str().unwrap();
                        if filter.is_match(strpath) {
                            let mut parents: Vec<_> = path
                                .iter_self_and_parents()
                                .take_while(|p| !processed_paths.contains(*p))
                                .collect();
                            parents.reverse();
                            for parent in parents {
                                let fdest = dest.join(parent);
                                try!(shallow_copy(&src.join(parent), &fdest,
                                        self.owner_uid, self.owner_gid)
                                    .context((&fpath, &fdest)));
                                processed_paths.insert(PathBuf::from(parent));
                            }
                        }
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
