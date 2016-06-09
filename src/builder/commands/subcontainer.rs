use std::path::{Path, PathBuf};
use std::fs::File;

use libmount::BindMount;

use quire::validate as V;
use config::read_config;
use config::containers::Container as Cont;
use version::short_version;
use container::mount::{remount_ro};
use container::util::{copy_dir};
use file_util::{create_dir, shallow_copy};
use path_util::ToRelative;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

use builder::error::StepError as E;

// Build Steps
#[derive(Debug)]
pub struct Container(String);
tuple_struct_decode!(Container);

impl Container {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(RustcDecodable, Debug)]
pub struct Build {
    pub container: String,
    pub source: PathBuf,
    pub path: Option<PathBuf>,
    pub temporary_mount: Option<PathBuf>,
}

impl Build {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("container", V::Scalar::new())
        .member("source".to_string(),
            V::Directory::new().is_absolute(true).default("/"))
        .member("path".to_string(),
            V::Directory::new().is_absolute(true).optional())
        .member("temporary_mount".to_string(),
            V::Directory::new().is_absolute(true).optional())
    }
}


#[derive(RustcDecodable, Debug)]
pub struct GitSource {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub enum Source {
    Git(GitSource),
    Container(String),
    Directory,
}

#[derive(RustcDecodable, Debug)]
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
            .is_absolute(false)
            .default("vagga.yaml"))
        .member("container", V::Scalar::new())
        .member("cache", V::Scalar::new().optional())
    }
}

pub fn build(binfo: &Build, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let ref name = binfo.container;
    let cont = guard.ctx.config.containers.get(name)
        .expect("Subcontainer not found");  // TODO
    if build {
        let version = try!(short_version(&cont, &guard.ctx.config)
            .map_err(|(s, e)| format!("step {}: {}", s, e)));
        let container = Path::new("/vagga/base/.roots")
            .join(format!("{}.{}", name, version));
        let path = container.join("root").join(binfo.source.rel());

        // Update container use when using it as subcontainer (fixes #267)
        File::create(Path::new(&container).join("last_use"))
            .map_err(|e| warn!("Can't write image usage info: {}", e)).ok();

        if let Some(ref dest_rel) = binfo.path {
            let dest = Path::new("/vagga/root")
                .join(dest_rel.rel());
            if path.is_dir() {
                try_msg!(copy_dir(&path, &dest, None, None),
                    "Error copying dir {p:?}: {err}", p=path);
            } else {
                try_msg!(shallow_copy(&path, &dest, None, None),
                    "Error copying dir {p:?}: {err}", p=path);
            }
        } else if let Some(ref dest_rel) = binfo.temporary_mount {
            let dest = Path::new("/vagga/root")
                .join(dest_rel.rel());
            try_msg!(create_dir(&dest, false),
                "Error creating destination dir: {err}");
            try!(BindMount::new(&path, &dest).mount());
            try!(remount_ro(&dest));
            guard.ctx.mounted.push(dest);
        }
    }
    Ok(())
}

fn real_copy(name: &String, cont: &Cont, guard: &mut Guard)
    -> Result<(), StepError>
{
    let version = try!(short_version(&cont, &guard.ctx.config)
        .map_err(|(s, e)| format!("step {}: {}", s, e)));
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

pub fn clone(name: &String, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let cont = guard.ctx.config.containers.get(name)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        try!(b.build(guard, false)
            .map_err(|e| E::SubStep(b.0.clone(), Box::new(e))));
    }
    if build {
        try!(real_copy(name, cont, guard));
    }
    Ok(())
}

fn find_config(cfg: &SubConfig, guard: &mut Guard)
    -> Result<Config, StepError>
{
    let path = match cfg.source {
        Source::Container(ref container) => {
            let cont = guard.ctx.config.containers.get(container)
                .expect("Subcontainer not found");  // TODO
            let version = try!(short_version(&cont, &guard.ctx.config)
                .map_err(|(s, e)| format!("step {}: {}", s, e)));
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
    Ok(try!(read_config(&path)))
}

pub fn subconfig(cfg: &SubConfig, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let subcfg = try!(find_config(cfg, guard));
    let cont = subcfg.containers.get(&cfg.container)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        try!(b.build(guard, build)
            .map_err(|e| E::SubStep(b.0.clone(), Box::new(e))));
    }
    Ok(())
}

impl BuildStep for Container {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let cont = try!(cfg.containers.get(&self.0)
            .ok_or(VersionError::ContainerNotFound(self.0.to_string())));
        for b in cont.setup.iter() {
            debug!("Versioning setup: {:?}", b);
            try!(b.hash(cfg, hash));
        }
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        clone(&self.0, guard, build)
    }
    fn is_dependent_on(&self) -> Option<&str> {
        Some(&self.0)
    }
}
impl BuildStep for Build {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let cont = try!(cfg.containers.get(&self.container)
            .ok_or(VersionError::ContainerNotFound(self.container.to_string())));
        for b in cont.setup.iter() {
            debug!("Versioning setup: {:?}", b);
            try!(b.hash(cfg, hash));
        }
        Ok(())
    }
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
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = match self.source {
            Source::Container(ref container) => {
                let cinfo = try!(cfg.containers.get(container)
                    .ok_or(VersionError::ContainerNotFound(container.clone())));
                let version = try!(short_version(&cinfo, cfg));
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
        let subcfg = try!(read_config(&path));
        let cont = try!(subcfg.containers.get(&self.container)
            .ok_or(VersionError::ContainerNotFound(self.container.to_string())));
        for b in cont.setup.iter() {
            debug!("Versioning setup: {:?}", b);
            try!(b.hash(cfg, hash));
        }
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        subconfig(self, guard, build)
    }
    fn is_dependent_on(&self) -> Option<&str> {
        match self.source {
            Source::Directory => None,
            Source::Container(ref name) => Some(name),
            Source::Git(ref _git) => None,
        }
    }
}
