use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;

use unshare::{Command};
use quire::validate as V;

use super::super::context::Context;
use process_util::{capture_stdout, set_fake_uidmap};
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

// Build Steps
#[derive(Debug)]
pub struct Sh(String);
tuple_struct_decode!(Sh);

impl Sh {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(Debug)]
pub struct Cmd(Vec<String>);
tuple_struct_decode!(Cmd);

impl Cmd {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(RustcDecodable, Debug)]
pub struct RunAs {
    pub user_id: u32,
    pub group_id: u32,
    pub supplementary_gids: Vec<u32>,
    pub external_user_id: Option<u32>,
    pub work_dir: PathBuf,
    pub script: String,
}

impl RunAs {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("user_id", V::Numeric::new().min(0).default(0))
        .member("group_id", V::Numeric::new().min(0).default(0))
        .member("external_user_id", V::Numeric::new().min(0).optional())
        .member("supplementary_gids", V::Sequence::new(V::Numeric::new()))
        .member("work_dir", V::Directory::new().default("/work"))
        .member("script", V::Scalar::new())
    }
}

#[derive(Debug)]
pub struct Depends(PathBuf);
tuple_struct_decode!(Depends);

impl Depends {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(Debug)]
pub struct Env(BTreeMap<String, String>);
tuple_struct_decode!(Env);

impl Env {
    pub fn config() -> V::Mapping<'static> {
        V::Mapping::new(
            V::Scalar::new(),
            V::Scalar::new())
    }
}


fn find_cmd<P:AsRef<Path>>(ctx: &Context, cmd: P)
    -> Result<PathBuf, StepError>
{
    let cmd = cmd.as_ref();
    let chroot = Path::new("/vagga/root");
    if let Some(paths) = ctx.environ.get("PATH") {
        for dir in paths[..].split(':') {
            let path = Path::new(dir);
            match path.strip_prefix("/") {
                Ok(rel_path) => {
                    if chroot.join(rel_path).join(cmd).exists() {
                        return Ok(path.join(cmd));
                    }
                },
                Err(_) => {
                    warn!("All items in PATH must be absolute, not {}",
                        path.display());
                    continue;
                }
            }
        }
        return Err(StepError::CommandNotFound(cmd.to_path_buf(), paths.clone()));
    }
    return Err(StepError::CommandNotFound(cmd.to_path_buf(),
        "-- empty PATH --".to_string()));
}

fn setup_command(ctx: &Context, cmd: &mut Command)
{
    cmd.chroot_dir("/vagga/root");
    cmd.env_clear();
    for (k, v) in ctx.environ.iter() {
        cmd.env(k, v);
    };
}

pub fn run_command_at_env(ctx: &mut Context, cmdline: &[String],
    path: &Path, env: &[(&str, &str)])
    -> Result<(), String>
{
    let cmdpath = if cmdline[0][..].starts_with("/") {
        PathBuf::from(&cmdline[0])
    } else {
        try!(find_cmd(ctx, &cmdline[0]))
    };

    let mut cmd = Command::new(&cmdpath);
    setup_command(ctx, &mut cmd);
    cmd.args(&cmdline[1..]);
    cmd.current_dir(path);
    for &(k, v) in env.iter() {
        cmd.env(k, v);
    }

    debug!("Running {:?}", cmd);

    match cmd.status() {
        Ok(ref s) if s.success() => {
            return Ok(());
        }
        Ok(s) => {
            return Err(format!("Command {:?} {}", cmd, s));
        }
        Err(e) => {
            return Err(format!("Couldn't run {:?}: {}", cmd, e));
        }
    }
}

pub fn run_command_at(ctx: &mut Context, cmdline: &[String], path: &Path)
    -> Result<(), String>
{
    run_command_at_env(ctx, cmdline, path, &[])
}

pub fn run_command(ctx: &mut Context, cmd: &[String])
    -> Result<(), String>
{
    return run_command_at_env(ctx, cmd, &Path::new("/work"), &[]);
}

pub fn capture_command<'x>(ctx: &mut Context, cmdline: &'x[String],
    env: &[(&str, &str)])
    -> Result<Vec<u8>, String>
{
    let cmdpath = if cmdline[0][..].starts_with("/") {
        PathBuf::from(&cmdline[0])
    } else {
        try!(find_cmd(ctx, &cmdline[0]))
    };

    let mut cmd = Command::new(&cmdpath);
    cmd.chroot_dir("/vagga/root");
    cmd.args(&cmdline[1..]);
    cmd.env_clear();
    for (k, v) in ctx.environ.iter() {
        cmd.env(k, v);
    }
    for &(k, v) in env.iter() {
        cmd.env(k, v);
    }
    capture_stdout(cmd)
}


pub fn command<P:AsRef<Path>>(ctx: &Context, cmdname: P)
    -> Result<Command, StepError>
{
    let cmdpath = cmdname.as_ref();
    let mut cmd = if cmdpath.is_absolute() {
        Command::new(&cmdpath)
    } else {
        Command::new(try!(find_cmd(ctx, cmdpath)))
    };

    cmd.current_dir("/work");
    cmd.chroot_dir("/vagga/root");
    cmd.env_clear();
    for (k, v) in ctx.environ.iter() {
        cmd.env(k, v);
    }
    Ok(cmd)
}

pub fn run(mut cmd: Command) -> Result<(), StepError> {
    debug!("Running {:?}", cmd);

    match cmd.status() {
        Ok(ref s) if s.success() => Ok(()),
        Ok(s) => Err(StepError::CommandFailed(Box::new(cmd), s)),
        Err(e) => Err(StepError::CommandError(Box::new(cmd), e)),
    }
}

impl BuildStep for Sh {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("Sh", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(run_command(&mut guard.ctx,
                &["/bin/sh".to_string(),
                  "-exc".to_string(),
                  self.0.to_string()]));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Depends {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = Path::new("/work").join(&self.0);
        hash.file(&path, None, None)
        .map_err(|e| VersionError::Io(e, path))
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


impl BuildStep for Cmd {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("Cmd", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(run_command(&mut guard.ctx, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Env {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        for (k, v) in &self.0 {
            hash.field(k, v);
        }
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        for (k, v) in &self.0 {
            guard.ctx.environ.insert(k.clone(), v.clone());
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for RunAs {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.text("user_id", &self.user_id);
        hash.text("group_id", &self.group_id);
        if let Some(euid) = self.external_user_id {
            hash.text("external_user_id", &euid);
        }
        for i in self.supplementary_gids.iter() {
            hash.text("supplementary_gids", &i);
        }
        hash.field("work_dir", self.work_dir.as_os_str().as_bytes());
        hash.field("script", &self.script);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let mut cmd = Command::new("/bin/sh");
            setup_command(&guard.ctx, &mut cmd);
            cmd.arg("-exc");
            cmd.arg(&self.script);
            cmd.current_dir(&Path::new("/work").join(&self.work_dir));

            let uid = self.user_id;
            let gid = self.group_id;
            if let Some(euid) = self.external_user_id {
                try!(set_fake_uidmap(&mut cmd, uid, euid));
            }
            cmd.uid(uid);
            cmd.gid(gid);
            cmd.groups(self.supplementary_gids.clone());

            run(cmd)
        } else {
            Ok(())
        }
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
