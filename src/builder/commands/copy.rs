use std::io::ErrorKind;
use std::fs::Metadata;
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


const DEFAULT_UMASK: u32 = 0o002;

const DIR_MODE: u32 = 0o777;
const FILE_MODE: u32 = 0o666;
const EXE_FILE_MODE: u32 = 0o777;
const EXE_CHECK_MASK: u32 = 0o100;


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
        hash_path(hash, &path, &filter)
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
    pub umask: u32,
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
        .member("umask", V::Numeric::new().min(0).max(0o777).default(
            DEFAULT_UMASK as i64))
    }
}

impl BuildStep for Copy {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let ref src = self.source;
        if src.starts_with("/work") {
            let filter = try!(Filter::new(&self.ignore_regex, &self.include_regex));
            try!(hash_path(hash, src, &filter));
        } else {
            // We don't version the files outside of the /work because
            // we believe they are result of the commands run above
            //
            // And we need already built container to version the files
            // inside the container which is ugly
        }
        hash.field("path", self.path.to_str().unwrap());
        if let Some(uid) = self.owner_uid {
            hash.field("owner_uid", uid.to_string());
        }
        if let Some(gid) = self.owner_gid {
            hash.field("owner_gid", gid.to_string());
        }
        hash.field("umask", format!("{:o}", self.umask));
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
            let typ = try!(src.symlink_metadata()
                .map_err(|e| StepError::Read(src.into(), e)));
            if typ.is_dir() {
                try!(create_dir_mode(dest, DIR_MODE & !self.umask)
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
                            let parents: Vec<_> = path
                                .iter_self_and_parents()
                                .take_while(|p| !processed_paths.contains(*p))
                                .collect();
                            for parent in parents.iter().rev() {
                                let fdest = dest.join(parent);
                                let fsrc = &src.join(parent);
                                let fsrc_stat = try!(fsrc.symlink_metadata()
                                    .map_err(|e| StepError::Read(src.into(), e)));
                                try!(shallow_copy(&fsrc, &fsrc_stat, &fdest,
                                        self.owner_uid, self.owner_gid,
                                        calc_mode(&fsrc_stat, self.umask))
                                    .context((&fpath, &fdest)));
                                processed_paths.insert(PathBuf::from(parent));
                            }
                        }
                    }
                    Ok(())
                }).map_err(StepError::ScanDir).and_then(|x| x));
            } else {
                try!(shallow_copy(&self.source, &typ, dest,
                        self.owner_uid, self.owner_gid,
                        calc_mode(&typ, self.umask))
                    .context((&self.source, dest)));
            }
            Ok(())
        })
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn calc_mode(stat: &Metadata, umask: u32)
    -> Option<u32>
{
    if stat.is_dir() {
        Some(DIR_MODE & !umask)
    } else if stat.is_file() {
        let orig_mode = stat.permissions().mode();
        if orig_mode & EXE_CHECK_MASK > 0 {
            Some(EXE_FILE_MODE & !umask)
        } else {
            Some(FILE_MODE & !umask)
        }
    } else {
        None
    }
}

fn hash_path(hash: &mut Digest, path: &Path, filter: &Filter)
    -> Result<(), VersionError>
{
    match path.symlink_metadata() {
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
                    let full_path = path.join(&cur_path);
                    let stat = try!(full_path.symlink_metadata()
                        .map_err(|e| VersionError::Io(e, PathBuf::from(&full_path))));
                    try!(hash_file_and_exe_mode(hash, &full_path, &stat));
                }
                Ok(())
            }).map_err(VersionError::ScanDir).and_then(|x| x));
        }
        Ok(ref meta) => {
            try!(hash_file_and_exe_mode(hash, path, meta));
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

fn hash_file_and_exe_mode(hash: &mut Digest, path: &Path, stat: &Metadata)
    -> Result<(), VersionError>
{
    if stat.is_file() {
        let is_exe = (stat.permissions().mode() & EXE_CHECK_MASK) > 0;
        hash.bool("is_executable", is_exe);
    }
    try!(hash.file(path, stat)
        .map_err(|e| VersionError::Io(e, path.into())));
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
