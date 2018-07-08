use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::os::unix::fs::{PermissionsExt, MetadataExt};

use quire::validate as V;
#[cfg(feature="containers")]
use libmount::{BindMount, Remount};
use quick_error::ResultExt;

use config::read_config;
use config::containers::Container as Cont;
#[cfg(feature="containers")] use version::short_version;
#[cfg(feature="containers")] use container::util::{copy_dir};
#[cfg(feature="containers")] use file_util::{Dir, ShallowCopy};
#[cfg(feature="containers")] use path_util::IterSelfAndParents;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

use builder::StepError as E;
#[cfg(feature="containers")]
use builder::dns::revert_name_files;
#[cfg(feature="containers")]
use builder::commands::copy::{create_path_filter, hash_path};
#[cfg(feature="containers")]
use builder::commands::copy::{DEFAULT_ATIME, DEFAULT_MTIME};
#[cfg(feature="containers")]
use builder::commands::copy::{hash_file_content};

// Build Steps
#[derive(Debug, Serialize, Deserialize)]
pub struct Container(String);

impl Container {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    pub container: String,
    pub source: PathBuf,
    pub path: Option<PathBuf>,
    pub temporary_mount: Option<PathBuf>,
    pub content_hash: bool,
    pub rules: Vec<String>,
}

impl Build {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("container", V::Scalar::new())
        .member("source".to_string(),
            V::Directory::new().absolute(true).default("/"))
        .member("path".to_string(),
            V::Directory::new().absolute(true).optional())
        .member("temporary_mount".to_string(),
            V::Directory::new().absolute(true).optional())
        .member("content_hash", V::Scalar::new().default(false))
        .member("rules", V::Sequence::new(V::Scalar::new()))
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GitSource {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Source {
    Git(GitSource),
    Container(String),
    Directory,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubConfig {
    pub source: Source,
    pub path: PathBuf,
    pub container: String,
    pub cache: Option<bool>,
}

impl SubConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("source", V::Enum::new()
            .option("Directory", V::Nothing)
            .option("Container", V::Scalar::new())
            .option("Git", V::Structure::new()
                .member("url", V::Scalar::new())
                .member("revision", V::Scalar::new().optional())
                .member("branch", V::Scalar::new().optional()))
            .optional()
            .default_tag("Directory"))
        .member("path".to_string(), V::Directory::new()
            .absolute(false)
            .default("vagga.yaml"))
        .member("container", V::Scalar::new())
        .member("cache", V::Scalar::new().optional())
    }
}

#[cfg(feature="containers")]
pub fn build(binfo: &Build, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let ref name = binfo.container;
    let cont = guard.ctx.config.containers.get(name)
        .expect("Subcontainer not found");  // TODO
    if !build {
        return Ok(())
    }
    let version = short_version(&cont, &guard.ctx.config)
        .map_err(|(s, e)| format!("step {}: {}", s, e))?;
    let container = Path::new("/vagga/base/.roots")
        .join(format!("{}.{}", name, version));
    let ref src = container.join("root")
        .join(binfo.source.strip_prefix("/").unwrap());

    // Update container use when using it as subcontainer (fixes #267)
    File::create(Path::new(&container).join("last_use"))
        .map_err(|e| warn!("Can't write image usage info: {}", e)).ok();

    if let Some(ref dest_rel) = binfo.path {
        let ref dest = Path::new("/vagga/root")
            .join(dest_rel.strip_prefix("/").unwrap());
        let ref typ = src.symlink_metadata()
            .map_err(|e| StepError::Read(src.into(), e))?;
        if typ.is_dir() {
            ShallowCopy::new(src, dest)
                .src_stat(typ)
                .copy()
                .context((src, dest))?;
            let filter = create_path_filter(&binfo.rules, Some(true),
                &None, &None, true)?;
            let mut processed_paths = HashSet::new();
            filter.walk(src, |iter| {
                for entry in iter {
                    let fpath = entry.path();
                    // We know that directory is inside
                    // the source
                    let path = fpath.strip_prefix(&src).unwrap();
                    let parents: Vec<_> = path
                        .iter_self_and_parents()
                        .take_while(|p| !processed_paths.contains(*p))
                        .collect();
                    for parent in parents.iter().rev() {
                        let ref fdest = dest.join(parent);
                        let ref fsrc = src.join(parent);
                        let ref fsrc_stat = fsrc.symlink_metadata()
                            .map_err(|e| StepError::Read(src.into(), e))?;
                        let mut cp = ShallowCopy::new(fsrc, fdest);
                        cp.src_stat(fsrc_stat);
                        cp.times(DEFAULT_ATIME, DEFAULT_MTIME);
                        cp.copy().context((fsrc, fdest))?;
                        processed_paths.insert(PathBuf::from(parent));
                    }
                }
                Ok(())
            }).map_err(StepError::PathFilter).and_then(|x| x)?;
        } else {
            try_msg!(ShallowCopy::new(&src, &dest).copy(),
                "Error copying file {p:?}: {err}", p=src);
        }
    } else if let Some(ref dest_rel) = binfo.temporary_mount {
        let dest = Path::new("/vagga/root")
            .join(dest_rel.strip_prefix("/").unwrap());
        try_msg!(Dir::new(&dest).create(),
            "Error creating destination dir: {err}");
        BindMount::new(&src, &dest).mount()?;
        Remount::new(&dest).bind(true).readonly(true).remount()?;
        guard.ctx.mounted.push(dest);
    }
    Ok(())
}

#[cfg(feature="containers")]
fn real_copy(name: &String, cont: &Cont, guard: &mut Guard)
    -> Result<(), StepError>
{
    let version = short_version(&cont, &guard.ctx.config)
        .map_err(|(s, e)| format!("step {}: {}", s, e))?;
    let container = format!("/vagga/base/.roots/{}.{}", name, version);

    // Update container use when using it as subcontainer (fixes #267)
    File::create(Path::new(&container).join("last_use"))
        .map_err(|e| warn!("Can't write image usage info: {}", e)).ok();

    let root = Path::new(&container).join("root");
    try_msg!(copy_dir(&root, &Path::new("/vagga/root"),
                      None, None),
        "Error copying dir {p:?}: {err}", p=root);
    Ok(())
}

#[cfg(feature="containers")]
pub fn clone(name: &String, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let cont = guard.ctx.config.containers.get(name)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        b.build(guard, false)
            .map_err(|e| E::SubStep(b.0.clone(), Box::new(e)))?;
    }
    if build {
        real_copy(name, cont, guard)?;
    }
    Ok(())
}

#[cfg(feature="containers")]
fn find_config(cfg: &SubConfig, guard: &mut Guard)
    -> Result<Config, StepError>
{
    let path = match cfg.source {
        Source::Container(ref container) => {
            let cont = guard.ctx.config.containers.get(container)
                .expect("Subcontainer not found");  // TODO
            let version = short_version(&cont, &guard.ctx.config)
                .map_err(|(s, e)| format!("step {}: {}", s, e))?;
            let container = Path::new("/vagga/base/.roots")
                .join(format!("{}.{}", container, version));

            // Update container use when using it as subcontainer (fixes #267)
            File::create(Path::new(&container).join("last_use"))
                .map_err(|e| warn!("Can't write image usage info: {}", e))
                .ok();

            container.join("root").join(&cfg.path)
        }
        Source::Git(ref _git) => {
            unimplemented!();
        }
        Source::Directory => {
            Path::new("/work").join(&cfg.path)
        }
    };
    Ok(read_config(&path.parent().expect("parent exists"), Some(&path), true)?)
}

#[cfg(feature="containers")]
pub fn subconfig(cfg: &SubConfig, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let subcfg = find_config(cfg, guard)?;
    let cont = subcfg.containers.get(&cfg.container)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        b.build(guard, build)
            .map_err(|e| E::SubStep(b.0.clone(), Box::new(e)))?;
    }
    Ok(())
}

impl BuildStep for Container {
    fn name(&self) -> &'static str { "Container" }
    #[cfg(feature="containers")]
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let cont = cfg.containers.get(&self.0)
            .ok_or_else(|| format_err!("container {:?} not found", self.0))?;
        for b in cont.setup.iter() {
            debug!("Versioning setup: {:?}", b);
            hash.command(b.name());
            b.hash(cfg, hash)?;
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        clone(&self.0, guard, build)?;
        revert_name_files()?;
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        Some(&self.0)
    }
}
impl BuildStep for Build {
    fn name(&self) -> &'static str { "Build" }
    #[cfg(feature="containers")]
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let cinfo = cfg.containers.get(&self.container)
            .ok_or_else(|| format_err!("Container {} not found",
                self.container))?;
        if self.content_hash {
            let version = short_version(&cinfo, cfg)
                .map_err(|(s, e)| format_err!("{}: {}", s, e))?;
            let root = Path::new("/vagga/base/.roots")
                .join(format!("{}.{}", self.container, version))
                .join("root");
            if !root.exists() {
                return Err(VersionError::New);
            }
            if let Some(ref dest_rel) = self.path {
                let filter = create_path_filter(&self.rules, Some(true),
                    &None, &None, false)?;
                let spath = self.source.strip_prefix("/")
                    .expect("absolute_source_path");
                hash_path(hash, &root.join(&spath), &filter, |h, p, st| {
                    h.field("filename", p);
                    h.field("mode", st.permissions().mode() & 0o7777);
                    h.field("uid", st.uid());
                    h.field("gid", st.gid());
                    hash_file_content(h, p, st)
                        .map_err(|e| VersionError::io(e, p))?;
                    Ok(())
                })?;
                hash.field("path", dest_rel);
            } else if let Some(_) = self.temporary_mount {
                unimplemented!("Build: combination of \
                    content-hash and temporary-mount are not supported yet");
            }
        } else {
            if !self.rules.is_empty() && self.temporary_mount.is_some() {
                unimplemented!("Build: combination of \
                    rules and temporary-mount are not supported yet");
            }
            for b in cinfo.setup.iter() {
                debug!("Versioning setup: {:?}", b);
                hash.command(b.name());
                b.hash(cfg, hash)?;
            }
            // TODO(tailhook) should we hash our params?!?!
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, do_build: bool)
        -> Result<(), StepError>
    {
        build(&self, guard, do_build)
    }
    fn is_dependent_on(&self) -> Option<&str> {
        Some(&self.container)
    }
}
impl BuildStep for SubConfig {
    fn name(&self) -> &'static str { "SubConfig" }
    #[cfg(feature="containers")]
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = match self.source {
            Source::Container(ref container) => {
                let cinfo = cfg.containers.get(container)
                    .ok_or_else(|| {
                        format_err!("container {:?} not found", container)
                    })?;
                let version = short_version(&cinfo, cfg)
                    .map_err(|(s, e)| format_err!("{}: {}", s, e))?;
                Path::new("/vagga/base/.roots")
                    .join(format!("{}.{}", container, version))
                    .join("root").join(&self.path)
            }
            Source::Git(ref _git) => {
                unimplemented!();
            }
            Source::Directory => {
                Path::new("/work").join(&self.path)
            }
        };
        if !path.exists() {
            return Err(VersionError::New);
        }
        let subcfg = read_config(
            path.parent().expect("has parent directory"),
            Some(&path), true)?;
        let cont = subcfg.containers.get(&self.container)
            .ok_or_else(|| {
                format_err!("container {:?} not found", self.container)
            })?;
        for b in cont.setup.iter() {
            debug!("Versioning setup: {:?}", b);
            hash.command(b.name());
            b.hash(cfg, hash)?;
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        subconfig(self, guard, build)?;
        revert_name_files()?;
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        match self.source {
            Source::Directory => None,
            Source::Container(ref name) => Some(name),
            Source::Git(ref _git) => None,
        }
    }
}
