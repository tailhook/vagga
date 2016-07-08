use std::io::ErrorKind;
use std::fs::{symlink_metadata};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{PermissionsExt, MetadataExt};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use libc::{uid_t, gid_t};
use quire::ast::{Ast, Tag};
use quire::validate as V;
use regex::{Regex, Error as RegexError};
use scan_dir::ScanDir;

use container::root::temporary_change_root;
use file_util::{create_dir_mode, shallow_copy};
use path_util::IterSelfAndParents;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use quick_error::ResultExt;


#[derive(RustcDecodable, Debug)]
pub struct Depends {
    pub path: PathBuf,
    pub ignore_regex: String,
    pub include_regex: Option<String>,
}

fn depends_parser(ast: Ast) -> BTreeMap<String, Ast> {
    match ast {
        Ast::Scalar(pos, _, style, value) => {
            let mut map = BTreeMap::new();
            map.insert("path".to_string(),
                Ast::Scalar(pos.clone(), Tag::NonSpecific, style, value));
            map
        },
        _ => unreachable!(),
    }
}

impl Depends {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
            .member("path", V::Scalar::new())
            .member("ignore_regex", V::Scalar::new().default(
                r#"(^|/)\.(git|hg|svn|vagga)($|/)|~$|\.bak$|\.orig$|^#.*#$"#))
            .member("include_regex", V::Scalar::new().optional())
            .parser(depends_parser)
    }
}

impl BuildStep for Depends {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let filter = try!(Filter::new(&self.ignore_regex, &self.include_regex));
        let path = Path::new("/work").join(&self.path);
        hash_path(hash, &path, &filter, None, None)
    }
    fn build(&self, _guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

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
}

impl BuildStep for Copy {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let ref src = self.source;
        if src.starts_with("/work") {
            let filter = try!(Filter::new(&self.ignore_regex, &self.include_regex));
            hash_path(hash, src, &filter, self.owner_uid, self.owner_gid)
        } else {
            // We don't version the files outside of the /work because
            // we believe they are result of the commands run above
            //
            // And we need already built container to version the files
            // inside the container which is ugly
            Ok(())
        }
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
                let filter = try!(Filter::new(&self.ignore_regex, &self.include_regex));
                let mut processed_paths = HashSet::new();
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

fn hash_path(hash: &mut Digest, path: &Path,
    filter: &Filter, owner_uid: Option<uid_t>, owner_gid: Option<gid_t>)
    -> Result<(), VersionError>
{
    match symlink_metadata(path) {
        Ok(ref meta) if meta.file_type().is_dir() => {
            try!(ScanDir::all().walk(path, |iter| {
                let mut all_paths = BTreeSet::new();
                for (entry, _) in iter {
                    let fpath = entry.path();
                    // We know that directory is inside
                    // the source
                    let rel_path = fpath.strip_prefix(path).unwrap();
                    // We know that path is decodable
                    let strpath = rel_path.to_str().unwrap();
                    if filter.is_match(strpath) {
                        for parent in rel_path.iter_self_and_parents() {
                            if all_paths.contains(parent) {
                                break;
                            }
                            all_paths.insert(PathBuf::from(parent));
                        }
                    }
                }
                for cur_path in all_paths {
                    hash.field("filename", cur_path.to_str().unwrap());
                    try!(hash.file(&path.join(&cur_path),
                                   owner_uid, owner_gid)
                         .map_err(|e| VersionError::Io(e, PathBuf::from(cur_path))));
                }
                Ok(())
            }).map_err(VersionError::ScanDir).and_then(|x| x));
        }
        Ok(_) => {
            try!(hash.file(path, owner_uid, owner_gid)
                 .map_err(|e| VersionError::Io(e, path.into())));
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            return Err(VersionError::New);
        }
        Err(e) => {
            return Err(VersionError::Io(e, path.into()));
        }
    }
    Ok(())
}

struct Filter {
    ignore_re: Option<Regex>,
    include_re: Option<Regex>,
}

impl Filter {
    fn new(ignore_regex: &String, include_regex: &Option<String>)
        -> Result<Filter, RegexError>
    {
        Ok(Filter {
            ignore_re: Some(try!(Regex::new(ignore_regex))),
            include_re: match *include_regex {
                Some(ref include_regex) => {
                    Some(try!(Regex::new(include_regex)))
                },
                None => None,
            },
        })
    }

    fn is_match(&self, s: &str) -> bool
    {
        self.ignore_re.as_ref().map_or(true, |r| !r.is_match(s))
            && self.include_re.as_ref().map_or(true, |r| r.is_match(s))
    }
}
