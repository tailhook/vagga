use std::path::Path;
use std::fs::File;

use unshare::Command;
use rustc_serialize::json::Json;

use super::super::context::{Context};
use super::generic::{run_command, capture_command};
use super::super::packages;
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use config::builders::NpmDepInfo;


fn scan_features(pkgs: &Vec<String>) -> Vec<packages::Package> {
    let mut res = vec!();
    res.push(packages::BuildEssential);
    res.push(packages::NodeJs);
    res.push(packages::NodeJsDev);
    res.push(packages::Npm);
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
    -> Result<(), String>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    try!(ensure_npm(distro, ctx, &scan_features(pkgs)[..]));
    let mut args = vec!(
        "/usr/bin/npm".to_string(),
        "install".to_string(),
        "--user=root".to_string(),
        "--cache=/tmp/npm-cache".to_string(),
        "--global".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, &args[..])
}

fn scan_dic(json: &Json, key: &str, cmd: &mut Command,
    features: &mut Vec<packages::Package>) -> Result<(), StepError>
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
                cmd.arg(format!("{}@{}", k, s));
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
    info: &NpmDepInfo)
    -> Result<(), StepError>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    let mut features = scan_features(&Vec::new());

    let mut cmd = try!(command(ctx, "/usr/bin/npm"));
    cmd.arg("install");
    cmd.arg("--global");
    cmd.arg("--user=root");
    cmd.arg("--cache=/tmp/npm-cache");

    let json = try!(File::open(&Path::new("/work").join(&info.file))
        .map_err(|e| format!("Error opening file {:?}: {}", info.file, e))
        .and_then(|mut f| Json::from_reader(&mut f)
        .map_err(|e| format!("Error parsing json {:?}: {}", info.file, e))));

    if info.package {
        try!(scan_dic(&json, "dependencies", &mut cmd, &mut features));
    }
    if info.dev {
        try!(scan_dic(&json, "devDependencies", &mut cmd, &mut features));
    }
    if info.peer {
        try!(scan_dic(&json, "peerDependencies", &mut cmd, &mut features));
    }
    if info.bundled {
        try!(scan_dic(&json, "bundledDependencies", &mut cmd, &mut features));
        try!(scan_dic(&json, "bundleDependencies", &mut cmd, &mut features));
    }
    if info.optional {
        try!(scan_dic(&json, "optionalDependencies", &mut cmd, &mut features));
    }

    try!(ensure_npm(distro, ctx, &features));

    run(cmd)
}

pub fn list(ctx: &mut Context) -> Result<(), String> {
    use std::fs::File;  // TODO(tailhook) migrate whole module
    use std::io::Write;  // TODO(tailhook) migrate whole module
    try!(capture_command(ctx, &[
            "/usr/bin/npm".to_string(),
            "ls".to_string(),
            "--global".to_string(),
        ], &[])
        .and_then(|out| {
            File::create("/vagga/container/npm-list.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping package list: {}", e))
        }));
    Ok(())
}
