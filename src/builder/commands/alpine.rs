use std::io::Write;
use std::fs::{copy};
use std::fs::File;
use std::path::Path;

use unshare::{Command, Stdio};

use super::super::super::file_util::create_dir;
use super::super::context::{BuildContext};
use super::super::context::Distribution::{Alpine};
use super::super::capsule;
use super::super::packages;
use process_util::capture_stdout;


pub static LATEST_VERSION: &'static str = "v3.1";


#[derive(Debug)]
pub struct AlpineInfo {
    pub version: String,
    pub base_setup: bool,
}


pub fn setup_base(ctx: &mut BuildContext, version: &String)
    -> Result<(), String>
{
    let base = if let Alpine(ref alpine) = ctx.distribution {
        alpine.base_setup
    } else {
        return Err(format!("Incompatible distribution: {:?}",
                           ctx.distribution));
    };
    if !base {
        try!(capsule::ensure_features(ctx, &[capsule::AlpineInstaller]));
        try_msg!(create_dir(&Path::new("/vagga/root/etc/apk"), true),
            "Error creating apk dir: {err}");
        // TODO(tailhook) use specified version instead of one in capsule
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
        if let Alpine(ref mut alpine) = ctx.distribution {
            alpine.base_setup = true;
        }
    }
    Ok(())
}


pub fn install(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    capsule::apk_run(&[
        "--root", "/vagga/root",
        "add",
        ], &pkgs[..])
}

pub fn remove(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    capsule::apk_run(&[
        "--root", "/vagga/root",
        "del",
        ], &pkgs[..])
}

pub fn finish(ctx: &mut BuildContext) -> Result<(), String>
{
    let pkgs = ctx.build_deps.clone().into_iter().collect();
    try!(remove(ctx, &pkgs));
    let mut cmd = Command::new("/vagga/bin/apk");
    cmd
        .stdin(Stdio::null())
        .env_clear()
        .arg("--root").arg("/vagga/root")
        .arg("-vv")
        .arg("info");
    try!(capture_stdout(cmd)
        .map_err(|e| format!("Error dumping package list: {}", e))
        .and_then(|out| {
            File::create("/vagga/container/alpine-packages.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping package list: {}", e))
        }));
    Ok(())
}

fn build_deps(pkg: packages::Package) -> Option<Vec<&'static str>> {
    match pkg {
        packages::BuildEssential => Some(vec!("build-base")),
        packages::Python2 => Some(vec!()),
        packages::Python2Dev => Some(vec!("python-dev")),
        packages::Python3 => None,
        packages::Python3Dev => None,
        packages::PipPy2 => None,
        packages::PipPy3 => None,
        packages::NodeJs => Some(vec!()),
        packages::NodeJsDev => Some(vec!("nodejs-dev")),
        packages::Npm => Some(vec!()),
        packages::Git => Some(vec!("git")),
        packages::Mercurial => Some(vec!("hg")),
    }
}

fn system_deps(pkg: packages::Package) -> Option<Vec<&'static str>> {
    match pkg {
        packages::BuildEssential => Some(vec!()),
        packages::Python2 => Some(vec!("python")),
        packages::Python2Dev => Some(vec!()),
        packages::Python3 => None,
        packages::Python3Dev => None,
        packages::PipPy2 => None,
        packages::PipPy3 => None,
        packages::NodeJs => Some(vec!("nodejs")),
        packages::NodeJsDev => Some(vec!()),
        packages::Npm => Some(vec!("nodejs")),  // Need duplicate?
        packages::Git => Some(vec!()),
        packages::Mercurial => Some(vec!()),
    }
}

pub fn ensure_packages(ctx: &mut BuildContext, features: &[packages::Package])
    -> Result<Vec<packages::Package>, String>
{
    let mut to_install = vec!();
    let mut unsupp = vec!();
    for i in features.iter() {
        if let Some(lst) = build_deps(*i) {
            for i in lst.into_iter() {
                if !ctx.packages.contains(i) {
                    if ctx.build_deps.insert(i.to_string()) {
                        to_install.push(i.to_string());
                    }
                }
            }
        } else {
            unsupp.push(*i);
            continue;
        }
        if let Some(lst) = system_deps(*i) {
            for i in lst.into_iter() {
                let istr = i.to_string();
                ctx.build_deps.remove(&istr);
                if ctx.packages.insert(istr.clone()) {
                    to_install.push(istr);
                }
            }
        } else {
            unsupp.push(*i);
            continue;
        }
    }
    if to_install.len() > 0 {
        try!(capsule::apk_run(&[
            "--root", "/vagga/root",
            "add",
            ], &to_install[..]));
    }
    return Ok(unsupp);
}
