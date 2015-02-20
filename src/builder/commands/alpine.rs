use std::io::ALL_PERMISSIONS;
use std::io::fs::{File, mkdir_recursive, copy};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::super::context::{BuildContext};
use super::super::context::Distribution::{Alpine, Unknown};
use super::super::capsule;
use super::super::packages;


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
                    ctx.build_deps.insert(i.to_string());
                    to_install.push(i.to_string());
                }
            }
        } else {
            unsupp.push(*i);
            continue;
        }
        if let Some(lst) = system_deps(*i) {
            for i in lst.into_iter() {
                let istr = i.to_string();
                if !ctx.packages.contains(&istr) {
                    ctx.build_deps.remove(&istr);
                }
                if !ctx.packages.contains(&istr) {
                    ctx.packages.insert(istr.clone());
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
            ], to_install.as_slice()));
    }
    return Ok(unsupp);
}
