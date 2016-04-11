use std::path::Path;
use std::fs::File;
use std::os::unix::io::{FromRawFd, AsRawFd};

use unshare::{Stdio};
use rustc_serialize::json::Json;

use super::super::context::{Context};
use super::super::packages;
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use config::builders::{NpmConfig, NpmDependencies};

impl Default for NpmConfig {
    fn default() -> NpmConfig {
        NpmConfig {
            install_node: true,
            npm_exe: "npm".to_string(),
        }
    }
}

fn scan_features(settings: &NpmConfig, pkgs: &Vec<String>)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);
    if settings.install_node {
        res.push(packages::NodeJs);
        res.push(packages::NodeJsDev);
        res.push(packages::Npm);
    }
    for name in pkgs.iter() {
        parse_feature(&name, &mut res);
    }
    return res;
}

pub fn parse_feature(info: &str, features: &mut Vec<packages::Package>) {
    // Note: the info is a package name/git-url in NpmInstall but it's just
    // a version number for NpmDependencies. That's how npm works.
    if info[..].starts_with("git://") {
        features.push(packages::Git);
    } // TODO(tailhook) implement whole a lot of other npm version kinds
}

pub fn ensure_npm(distro: &mut Box<Distribution>, ctx: &mut Context,
    features: &[packages::Package])
    -> Result<(), StepError>
{
    packages::ensure_packages(distro, ctx, features)
}

pub fn npm_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), StepError>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    let features = scan_features(&ctx.npm_settings, pkgs);
    try!(ensure_npm(distro, ctx, &features));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = try!(command(ctx, &ctx.npm_settings.npm_exe));
    cmd.arg("install");
    cmd.arg("--global");
    cmd.arg("--user=root");
    cmd.arg("--cache=/tmp/npm-cache");
    cmd.args(pkgs);
    run(cmd)
}

fn scan_dic(json: &Json, key: &str,
    packages: &mut Vec<String>, features: &mut Vec<packages::Package>)
    -> Result<(), StepError>
{
    match json.find(key) {
        Some(&Json::Object(ref ob)) => {
            for (k, v) in ob {
                if !v.is_string() {
                    return Err(StepError::Compat(format!(
                        "Package {:?} has wrong version {:?}", k, v)));
                }
                let s = v.as_string().unwrap();
                parse_feature(&s, features);
                packages.push(format!("{}@{}", k, s));
                // TODO(tailhook) check the feature
            }
            Ok(())
        }
        None => {
            Ok(())
        }
        Some(_) => {
            Err(StepError::Compat(format!(
                "The {:?} is not a mapping (JSON object)", key)))
        }
    }
}

pub fn npm_deps(distro: &mut Box<Distribution>, ctx: &mut Context,
    info: &NpmDependencies)
    -> Result<(), StepError>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    let mut features = scan_features(&ctx.npm_settings, &Vec::new());

    let json = try!(File::open(&Path::new("/work").join(&info.file))
        .map_err(|e| format!("Error opening file {:?}: {}", info.file, e))
        .and_then(|mut f| Json::from_reader(&mut f)
        .map_err(|e| format!("Error parsing json {:?}: {}", info.file, e))));
    let mut packages = vec![];

    if info.package {
        try!(scan_dic(&json, "dependencies", &mut packages, &mut features));
    }
    if info.dev {
        try!(scan_dic(&json, "devDependencies", &mut packages, &mut features));
    }
    if info.peer {
        try!(scan_dic(&json, "peerDependencies",
            &mut packages, &mut features));
    }
    if info.bundled {
        try!(scan_dic(&json, "bundledDependencies",
            &mut packages, &mut features));
        try!(scan_dic(&json, "bundleDependencies",
            &mut packages, &mut features));
    }
    if info.optional {
        try!(scan_dic(&json, "optionalDependencies",
            &mut packages, &mut features));
    }

    try!(ensure_npm(distro, ctx, &features));

    if packages.len() == 0 {
        return Ok(());
    }

    let mut cmd = try!(command(ctx, &ctx.npm_settings.npm_exe));
    cmd.arg("install");
    cmd.arg("--global");
    cmd.arg("--user=root");
    cmd.arg("--cache=/tmp/npm-cache");
    cmd.args(&packages);
    run(cmd)
}

pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let path = Path::new("/vagga/container/npm-list.txt");
    let file = try!(File::create(&path)
        .map_err(|e| StepError::Write(path.to_path_buf(), e)));
    let mut cmd = try!(command(ctx, &ctx.npm_settings.npm_exe));
    cmd.arg("ls");
    cmd.arg("--global");
    // TODO(tailhook) fixme in rust 1.6. as_raw_fd -> into_raw_fd
    cmd.stdout(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) });
    run(cmd)
}
