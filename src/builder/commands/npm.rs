use super::super::context::{BuildContext};
use super::generic::run_command;
use super::super::packages;
use super::super::context::Distribution as Distr;
use super::alpine;


pub fn scan_features(pkgs: &Vec<String>) -> Vec<packages::Package> {
    let mut res = vec!();
    res.push(packages::BuildEssential);
    res.push(packages::NodeJs);
    res.push(packages::NodeJsDev);
    res.push(packages::Npm);
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git://") {
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
    try!(ensure_npm(ctx, scan_features(pkgs).as_slice()));
    let mut args = vec!(
        "/usr/bin/npm".to_string(),
        "install".to_string(),
        "--user=root".to_string(),
        "--cache=/tmp/npm-cache".to_string(),
        "--global".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}
