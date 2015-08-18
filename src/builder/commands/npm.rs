use super::super::context::{BuildContext};
use super::generic::{run_command, capture_command};
use super::super::packages;

use std::path::Path;


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

pub fn ensure_npm(ctx: &mut BuildContext, features: &[packages::Package])
    -> Result<(), String>
{
    packages::ensure_packages(ctx, features)
}

pub fn npm_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    try!(ensure_npm(ctx, &scan_features(pkgs)[..]));
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

pub fn list(ctx: &mut BuildContext) -> Result<(), String> {
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
