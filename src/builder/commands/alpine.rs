use std::io::ALL_PERMISSIONS;
use std::rand::{thread_rng, Rng};
use std::io::fs::{File, mkdir_recursive};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::super::context::{BuildContext};
use super::super::context::Distribution::{Alpine, Unknown};
use super::super::dev::RevControl;
use super::pip::PipFeatures as Pip;
use super::npm::NpmFeatures as Npm;

static MIRRORS: &'static str = include_str!("../../../alpine/MIRRORS.txt");

pub static LATEST_VERSION: &'static str = "v3.1";


#[derive(Show)]
pub struct AlpineInfo {
    pub mirror: String,
    pub version: String,
}


pub fn apk_run(args: &[&str], packages: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("/vagga/bin/apk");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
        .env("PATH", "/vagga/bin")
        .args(args)
        .args(packages);
    debug!("Running APK {}", cmd);
    return match cmd.output()
        .map_err(|e| format!("Can't run apk: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => Ok(()),
        Ok(val) => Err(format!("Apk exited with status: {}", val)),
        Err(x) => Err(format!("Error running tar: {}", x)),
    }
}

pub fn choose_mirror() -> String {
    let repos = MIRRORS.split('\n').collect::<Vec<&str>>();
    let mirror = thread_rng().choose(repos.as_slice())
        .expect("At least one mirror should work");
    debug!("Chosen mirror {}", mirror);
    return mirror.to_string();
}

pub fn setup_base(ctx: &mut BuildContext, version: &String)
    -> Result<(), String>
{
    let apk_dir = Path::new("/vagga/root/etc/apk");
    let mirror = match ctx.distribution {
        Alpine(ref distr) => &distr.mirror,
        _ => return Err(format!("Conflicting distribution")),
    };

    try!(mkdir_recursive(&apk_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));

    try!(File::create(&Path::new("/vagga/root/etc/apk/repositories"))
         .and_then(|mut f|
            writeln!(&mut f, "{}{}/main", mirror, version))
        .map_err(|e| format!("Error creating apk repo: {}", e)));
    try!(apk_run(&[
        "--allow-untrusted",
        "--update-cache",
        "--root", "/vagga/root",
        "--initdb",
        "add",
        "alpine-base",
        ], &[]));
    Ok(())
}


pub fn install(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "add",
        ], pkgs.as_slice())
}

pub fn remove(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "del",
        ], pkgs.as_slice())
}

pub fn finish(ctx: &mut BuildContext) -> Result<(), String>
{
    let pkgs = ctx.build_deps.clone().into_iter().collect();
    remove(ctx, &pkgs)
}

pub fn ensure_npm(ctx: &mut BuildContext, features: &[Npm])
    -> Result<Path, String>
{
    let mut packages = vec!("nodejs".to_string());
    ctx.packages.extend(packages.clone().into_iter());
    for i in features.iter() {
        let dep = match *i {
            Npm::Dev => "nodejs-dev".to_string(),
            Npm::Npm => continue,
            Npm::Rev(name) => revcontrol_package(name),
        };
        if !ctx.packages.contains(&dep) {
            if ctx.build_deps.insert(dep.clone()) {
                packages.push(dep);
            }
        }
    }
    try!(apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "add",
        ], packages.as_slice()));
    return Ok(Path::new("/usr/bin/npm"));
}

pub fn revcontrol_package(name: RevControl) -> String {
    match name {
        RevControl::Git => "git".to_string(),
        RevControl::Hg => "hg".to_string(),
    }
}

pub fn ensure_pip(ctx: &mut BuildContext, ver: u8, features: &[Pip])
    -> Result<Path, String>
{
    if ver != 2 {
        return Err(format!("Python {} is not supported", ver));
    }
    let mut packages = vec!(
        (if ver == 2 { "python" } else { "python3" }).to_string(),
        );
    ctx.packages.extend(packages.clone().into_iter());
    for i in features.iter() {
        let dep = match *i {
            Pip::Dev => (if ver == 2 { "python-dev" }
                         else { "python3-dev" }).to_string(),
            Pip::Pip => (if ver == 2 { "py-pip" }
                         else { "py3-pip" }).to_string(),
            Pip::Rev(name) => "git".to_string(),
        };
        if !ctx.packages.contains(&dep) {
            if ctx.build_deps.insert(dep.clone()) {
                packages.push(dep);
            }
        }
    }
    try!(apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "add",
        ], packages.as_slice()));
    return Ok(Path::new("/usr/bin/pip"));
}
