use super::super::context::{Context};
use super::generic::{run_command, capture_command};
use super::super::packages;

use std::path::Path;

use builder::error::StepError;
use builder::distrib::Distribution;


pub fn scan_features(pkgs: &Vec<String>) -> Vec<packages::Package> {
    let mut res = vec!();
    res.push(packages::BuildEssential);
    res.push(packages::NodeJs);
    res.push(packages::NodeJsDev);
    res.push(packages::Npm);
    for name in pkgs.iter() {
        if name[..].starts_with("git://") {
            res.push(packages::Git);
        } // Does npm support mercurial?
    }
    return res;
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
