use std::io::ALL_PERMISSIONS;
use std::io::fs::{File, mkdir_recursive, copy};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::super::context::{BuildContext};
use super::super::context::Distribution::{Alpine, Unknown};
use super::super::dev::RevControl;
use super::super::capsule;
use super::pip::PipFeatures as Pip;
use super::npm::NpmFeatures as Npm;


pub static LATEST_VERSION: &'static str = "v3.1";


#[derive(Show)]
pub struct AlpineInfo {
    pub version: String,
}


pub fn setup_base(ctx: &mut BuildContext, version: &String)
    -> Result<(), String>
{
    try!(capsule::ensure_features(ctx, &[capsule::AlpineInstaller]));
    try!(mkdir_recursive(&Path::new("/vagga/root/etc/apk"), ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));
    try!(copy(
        &Path::new("/etc/apk/repositories"),  // Same mirror as in capsule
        &Path::new("/vagga/root/etc/apk/repositories"))
        .map_err(|e| format!("Error creating apk repo: {}", e)));
    try!(capsule::apk_run(&[
        "--update-cache",
        "--keys-dir=/etc/apk/keys",  // Use keys from capsule
        "--root=/vagga/root",
        "--initdb",
        "add",
        "alpine-base",
        ], &[]));
    Ok(())
}


pub fn install(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    capsule::apk_run(&[
        "--root", "/vagga/root",
        "add",
        ], pkgs.as_slice())
}

pub fn remove(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    capsule::apk_run(&[
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
    try!(capsule::apk_run(&[
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
    try!(capsule::apk_run(&[
        "--root", "/vagga/root",
        "add",
        ], packages.as_slice()));
    return Ok(Path::new("/usr/bin/pip"));
}
