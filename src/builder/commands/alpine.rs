use std::io::ALL_PERMISSIONS;
use std::rand::{task_rng, Rng};
use std::io::fs::{File, mkdir_recursive};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::super::context::{BuildContext, Alpine, Unknown};
use super::super::dev;
use super::pip;
use super::npm;

static MIRRORS: &'static str = include_str!("../../../alpine/MIRRORS.txt");

pub static LATEST_VERSION: &'static str = "v3.1";


#[deriving(Show)]
pub struct AlpineInfo {
    mirror: String,
    version: String,
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

pub fn setup_base(ctx: &mut BuildContext, version: &String)
    -> Result<(), String>
{
    if let Unknown = ctx.distribution {
    } else {
        return Err(format!("Conflicting distribution"));
    };
    let apk_dir = Path::new("/vagga/root/etc/apk");
    let repos = MIRRORS.split('\n').collect::<Vec<&str>>();
    let mirror = task_rng().choose(repos.as_slice())
        .expect("At least one mirror should work");
    debug!("Chosen mirror {}", mirror);

    try!(mkdir_recursive(&Path::new("/vagga/root/etc/apk/cache"),
                         ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));
    try!(ctx.add_cache_dir(Path::new("/etc/apk/cache"),
                           "alpine-cache".to_string()));
    try!(mkdir_recursive(&apk_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));

    try!(File::create(&Path::new("/vagga/root/etc/apk/repositories"))
         .and_then(|mut f|
            writeln!(f, "{}{}/main", mirror, version))
        .map_err(|e| format!("Error creating apk repo: {}", e)));
    try!(apk_run(&[
        "--allow-untrusted",
        "--update-cache",
        "--root", "/vagga/root",
        "--initdb",
        "add",
        "alpine-base",
        ], &[]));
    ctx.distribution = Alpine(AlpineInfo {
        mirror: mirror.to_string(),
        version: version.to_string(),
    });
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

pub fn ensure_npm(ctx: &mut BuildContext, features: &[npm::NpmFeatures])
    -> Result<Path, String>
{
    let mut packages = vec!("nodejs".to_string());
    ctx.packages.extend(packages.clone().into_iter());
    for i in features.iter() {
        let dep = match *i {
            npm::Dev => "nodejs-dev".to_string(),
            npm::Npm => continue,
            npm::Rev(name) => revcontrol_package(name),
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

pub fn revcontrol_package(name: dev::RevControl) -> String {
    match name {
        dev::Git => "git".to_string(),
        dev::Hg => "hg".to_string(),
    }
}

pub fn ensure_pip(ctx: &mut BuildContext, ver: u8,
    features: &[pip::PipFeatures])
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
            pip::Dev => (if ver == 2 { "python-dev" }
                         else { "python3-dev" }).to_string(),
            pip::Pip => (if ver == 2 { "py-pip" }
                         else { "py3-pip" }).to_string(),
            pip::Rev(name) => "git".to_string(),
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
