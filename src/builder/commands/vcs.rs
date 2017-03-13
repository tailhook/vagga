use std::fs::rename;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};

use quire::validate as V;
use builder::commands::subcontainer::GitSource;
use capsule::packages as capsule;
use super::super::context::Context;
use super::generic::run_command_at;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use process_util::run_success;


#[derive(RustcDecodable, Debug)]
pub struct Git {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
    pub path: PathBuf,
}

impl Git {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("revision", V::Scalar::new().optional())
        .member("branch", V::Scalar::new().optional())
        .member("path", V::Directory::new().absolute(true))
    }
}

#[derive(RustcDecodable, Debug)]
pub struct GitInstall {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
    pub subdir: PathBuf,
    pub script: String,
}

impl GitInstall {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("revision", V::Scalar::new().optional())
        .member("branch", V::Scalar::new().optional())
        .member("subdir", V::Directory::new()
            .default(".").absolute(false))
        .member("script", V::Scalar::new()
                .default("./configure --prefix=/usr\n\
                          make\n\
                          make install\n"))
    }
}


fn git_cache(url: &String) -> Result<PathBuf, String> {
    let dirname = url.replace("%", "%25").replace("/", "%2F");
    let cache_path = Path::new("/vagga/cache/git").join(&dirname);
    if cache_path.is_dir() {
        let mut cmd = Command::new("/usr/bin/git");
        cmd.stdin(Stdio::null());
        cmd.arg("-C").arg(&cache_path);
        cmd.arg("fetch");
        cmd.current_dir(&cache_path);
        run_success(cmd)?;
    } else {
        let tmppath = cache_path.with_file_name(".tmp.".to_string() + &dirname);
        let mut cmd = Command::new("/usr/bin/git");
        cmd.stdin(Stdio::null());
        cmd.arg("clone").arg("--bare");
        cmd.arg(url).arg(&tmppath);
        run_success(cmd)?;
        rename(&tmppath, &cache_path)
            .map_err(|e| format!("Error renaming cache dir: {}", e))?;
    }
    Ok(cache_path)
}

fn git_checkout(cache_path: &Path, dest: &Path,
    revision: &Option<String>, branch: &Option<String>)
    -> Result<(), String>
{
    let mut cmd = Command::new("/usr/bin/git");
    cmd.stdin(Stdio::null());
    cmd.arg("--git-dir").arg(cache_path);
    cmd.arg("--work-tree").arg(dest);
    cmd.arg("reset").arg("--hard");
    if let &Some(ref rev) = revision {
        cmd.arg(&rev);
    } else if let &Some(ref branch) = branch {
        cmd.arg(&branch);
    } else {
    }
    run_success(cmd)?;
    Ok(())
}


pub fn git_command(ctx: &mut Context, git: &Git) -> Result<(), String>
{
    capsule::ensure(&mut ctx.capsule, &[capsule::Git])?;
    let dest = PathBuf::from("/vagga/root")
        .join(&git.path.strip_prefix("/").unwrap());
    let cache_path = git_cache(&git.url)?;
    create_dir_all(&dest)
         .map_err(|e| format!("Error creating dir: {}", e))?;
    git_checkout(&cache_path, &dest, &git.revision, &git.branch)?;
    Ok(())
}

pub fn git_install(ctx: &mut Context, git: &GitInstall)
    -> Result<(), String>
{
    capsule::ensure(&mut ctx.capsule, &[capsule::Git])?;
    let cache_path = git_cache(&git.url)?;
    let tmppath = Path::new("/vagga/root/tmp")
        .join(cache_path.file_name().unwrap());
    create_dir_all(&tmppath)
         .map_err(|e| format!("Error creating dir: {}", e))?;
    git_checkout(&cache_path, &tmppath, &git.revision, &git.branch)?;
    let workdir = PathBuf::from("/")
        .join(tmppath.strip_prefix("/vagga/root").unwrap())
        .join(&git.subdir);
    return run_command_at(ctx, &[
        "/bin/sh".to_string(),
        "-exc".to_string(),
        git.script.to_string()],
        &workdir);
}

#[allow(unused)]
pub fn fetch_git_source(capsule: &mut capsule::State, git: &GitSource)
    -> Result<(), String>
{
    capsule::ensure(capsule, &[capsule::Git])?;
    let cache_path = git_cache(&git.url)?;
    let dest = Path::new("/vagga/sources")
        .join(cache_path.file_name().unwrap());
    create_dir_all(&dest)
         .map_err(|e| format!("Error creating dir: {}", e))?;
    git_checkout(&cache_path, &dest, &git.revision, &git.branch)?;
    Ok(())
}

impl BuildStep for Git {
    fn name(&self) -> &'static str { "Git" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("url", &self.url);
        hash.opt_field("revision", &self.revision);
        hash.opt_field("branch", &self.branch);
        hash.field("path", &self.path);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            git_command(&mut guard.ctx, &self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for GitInstall {
    fn name(&self) -> &'static str { "GitInstall" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("url", &self.url);
        hash.opt_field("revision", &self.revision);
        hash.opt_field("branch", &self.branch);
        hash.field("subdir", &self.subdir);
        hash.field("script", &self.script);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            git_install(&mut guard.ctx, &self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
