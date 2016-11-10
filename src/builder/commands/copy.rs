use std::io::{self, ErrorKind};
use std::fs::{File, Metadata, read_link};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{PermissionsExt, MetadataExt};
use std::os::unix::ffi::OsStrExt;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use libc::{uid_t, gid_t};
use quire::ast::{Ast, Tag};
use quire::validate as V;
use regex::{Regex, Error as RegexError};
use scan_dir::{ScanDir, Error as ScanDirError};

use container::root::temporary_change_root;
use file_util::shallow_copy;
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
        let path = Path::new("/work").join(&self.path);
        let filter = Filter::new(
            &self.ignore_regex, &self.include_regex)?;
        hash_path(hash, &path, &filter, |h, p, st| {
            h.field("filename", p.as_os_str().as_bytes());
            // We hash only executable flag for files
            // as mode depends on the host system umask
            if st.is_file() {
                let mode = st.permissions().mode();
                let is_executable = mode & EXE_CHECK_MASK > 0;
                h.bool("is_executable", is_executable);
            }
            hash_file_content(h, p, st)
                .map_err(|e| VersionError::Io(e, PathBuf::from(p)))?;
            Ok(())
        })?;
        Ok(())
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
    pub preserve_permissions: bool,
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
        .member("preserve_permissions", V::Scalar::new().default(false))
    }

    fn calc_mode(&self, stat: &Metadata) -> Option<u32> {
        if stat.file_type().is_symlink() {
            // ignore as we do not set permissions for symlinks
            return None;
        }
        if self.preserve_permissions {
            Some(stat.permissions().mode())
        } else {
            let base_mode = if stat.is_dir() {
                DIR_MODE
            } else {
                if stat.permissions().mode() & EXE_CHECK_MASK > 0 {
                    EXE_FILE_MODE
                } else {
                    FILE_MODE
                }
            };
            Some(base_mode & !self.umask)
        }
    }
}

impl BuildStep for Copy {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let ref src = self.source;
        if src.starts_with("/work") {
            let filter = Filter::new(
                &self.ignore_regex, &self.include_regex)?;
            hash_path(hash, src, &filter, |h, p, st| {
                h.field("filename", p.as_os_str().as_bytes());
                if let Some(mode) = self.calc_mode(st) {
                    h.text("mode", mode);
                }
                h.text("uid", self.owner_uid.unwrap_or(st.uid()));
                h.text("gid", self.owner_gid.unwrap_or(st.gid()));
                hash_file_content(h, p, st)
                    .map_err(|e| VersionError::Io(e, PathBuf::from(p)))?;
                Ok(())
            })?;
            hash.field("path", self.path.to_str().unwrap());
        } else {
            // We don't version the files outside of the /work because
            // we believe they are result of the commands run above
            //
            // And we need already built container to version the files
            // inside the container which is ugly
            hash.field("source", src.to_str().unwrap());
            hash.field("path", self.path.to_str().unwrap());
            if !self.preserve_permissions {
                if let Some(uid) = self.owner_uid {
                    hash.field("owner_uid", uid.to_string());
                }
                if let Some(gid) = self.owner_gid {
                    hash.field("owner_gid", gid.to_string());
                }
                hash.field("umask", self.umask.to_string());
            }
            hash.bool("preserve_permissions", self.preserve_permissions);
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
            let ref dest = self.path;
            let ref typ = src.symlink_metadata()
                .map_err(|e| StepError::Read(src.into(), e))?;
            if typ.is_dir() {
                shallow_copy(src, typ, dest,
                        self.owner_uid, self.owner_gid,
                        self.calc_mode(typ))
                    .context((src, dest))?;
                let filter = Filter::new(&self.ignore_regex, &self.include_regex)?;
                let mut processed_paths = HashSet::new();
                ScanDir::all().walk(src, |iter| {
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
                                let ref fdest = dest.join(parent);
                                let ref fsrc = src.join(parent);
                                let ref fsrc_stat = fsrc.symlink_metadata()
                                    .map_err(|e| StepError::Read(src.into(), e))?;
                                shallow_copy(fsrc, fsrc_stat, fdest,
                                        self.owner_uid, self.owner_gid,
                                        self.calc_mode(fsrc_stat))
                                     .context((fsrc, fdest))?;
                                processed_paths.insert(PathBuf::from(parent));
                            }
                        }
                    }
                    Ok(())
                }).map_err(StepError::ScanDir).and_then(|x| x)?;
            } else {
                shallow_copy(src, typ, dest,
                        self.owner_uid, self.owner_gid,
                        self.calc_mode(typ))
                    .context((src, dest))?;
            }
            Ok(())
        })
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn hash_path<F>(hash: &mut Digest, path: &Path, filter: &Filter, hash_file: F)
    -> Result<(), VersionError>
    where F: Fn(&mut Digest, &Path, &Metadata) -> Result<(), VersionError>
{
    match path.symlink_metadata() {
        Ok(ref meta) if meta.file_type().is_dir() => {
            hash_file(hash, path, meta)?;
            let all_rel_paths = get_sorted_rel_paths(path, &filter)?;
            for rel_path in &all_rel_paths {
                let ref abs_path = path.join(rel_path);
                let stat = abs_path.symlink_metadata()
                    .map_err(|e| VersionError::Io(e, PathBuf::from(abs_path)))?;
                hash_file(hash, abs_path, &stat)?;
            }
        }
        Ok(ref meta) => {
            hash_file(hash, path, meta)?;
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

fn get_sorted_rel_paths(path: &Path, filter: &Filter)
    -> Result<BTreeSet<PathBuf>, Vec<ScanDirError>>
{
    ScanDir::all().walk(path, |iter| {
        let mut all_rel_paths = BTreeSet::new();
        for (entry, _) in iter {
            let fpath = entry.path();
            // We know that directory is inside
            // the path
            let rel_path = fpath.strip_prefix(path).unwrap();
            // We know that rel_path is decodable
            let strpath = rel_path.to_str().unwrap();
            if filter.is_match(strpath) {
                for parent in rel_path.iter_self_and_parents() {
                    if !all_rel_paths.contains(parent) {
                        all_rel_paths.insert(PathBuf::from(parent));
                    }
                }
            }
        }
        all_rel_paths
    })
}

fn hash_file_content(hash: &mut Digest, path: &Path, stat: &Metadata)
    -> Result<(), io::Error>
{
    if stat.file_type().is_file() {
        let mut file = File::open(&path)?;
        hash.stream(&mut file)?;
    } else if stat.file_type().is_symlink() {
        let data = read_link(path)?;
        hash.input(data.as_os_str().as_bytes());
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
            ignore_re: Some(Regex::new(ignore_regex)?),
            include_re: match *include_regex {
                Some(ref include_regex) => {
                    Some(Regex::new(include_regex)?)
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
