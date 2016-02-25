use std::path::Path;
use std::fs::File;
use std::io::Write;

use unshare::{Command};

use super::super::context::{Context};
use super::super::packages;
use super::generic::run_command;
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use builder::commands::ubuntu::{self, Ubuntu};
use config::builders::{ComposerSettings, ComposerReqInfo, AptKey, UbuntuRepoInfo};
use process_util::capture_stdout;

const HHVM_APT_KEY: &'static str = "5a16e7281be7a449";
const HHVM_REPO_URL: &'static str = "http://dl.hhvm.com/ubuntu";


impl Default for ComposerSettings {
    fn default() -> Self {
        ComposerSettings {
            engine: "php".to_owned(),
            engine_exe: None,
        }
    }
}

fn scan_features(settings: &ComposerSettings, _prefer_dist: bool)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::Composer);
    if settings.engine == "php" {
        res.push(packages::PHP);
    } else {
        res.push(packages::HHVM);
    }
    // Probably it's not it worth trying to figure out whether we need Git or Mercurial and it is
    // more likely that a php project is using Git, therefore it is reasonable to simply assume we
    // always need Git
    res.push(packages::Git);
    return res;
}

fn composer_cmd(ctx: &mut Context) -> Result<Command, StepError> {
    let mut cmd =
        if let Some(ref exe) = ctx.composer_settings.engine_exe {
            try!(command(ctx, exe))
        } else {
            let mut cmd = try!(command(ctx, "/usr/bin/env"));
            if ctx.composer_settings.engine == "php" {
                cmd.arg("php");
            } else if ctx.composer_settings.engine == "hhvm" {
                cmd.arg("hhvm");
            } else {
                unreachable!();
            }
            cmd
        };

    cmd.arg("/tmp/composer.phar");

    Ok(cmd)
}

pub fn composer_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), StepError>
{
    let features = scan_features(&ctx.composer_settings, true);

    try!(packages::ensure_packages(distro, ctx, &features));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = try!(composer_cmd(ctx));
    cmd.args(&["global", "require", "--prefer-dist"]);
    cmd.args(pkgs);
    run(cmd)
}

pub fn composer_requirements(distro: &mut Box<Distribution>, ctx: &mut Context,
    info: &ComposerReqInfo)
    -> Result<(), StepError>
{
    let prefer_dist = info.prefer.as_ref().map(|p| p == "dist").unwrap_or(false);
    let features = scan_features(&ctx.composer_settings, prefer_dist);

    try!(packages::ensure_packages(distro, ctx, &features));

    let mut cmd = try!(composer_cmd(ctx));

    if info.update { cmd.arg("update"); }
    else { cmd.arg("install"); }

    if !info.dev { cmd.arg("--no-dev"); }
    if info.optimize_autoload { cmd.arg("--optimize-autoload"); }

    match info.prefer {
        Some(ref p) if p == "dist" => { cmd.arg("--prefer-dist"); }
        Some(ref p) if p == "source" => { cmd.arg("--prefer-source"); }
        _ => {}
    }

    run(cmd)
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
    ctx.add_ensure_dir(Path::new("/tmp/composer/vendor"));
    let args = vec!(
        "/bin/ln".to_owned(),
        "-s".to_owned(),
        "/usr/lib/composer/vendor".to_owned(),
        "/composer".to_owned());
    try!(run_command(ctx, &args));

    try!(ctx.add_cache_dir(Path::new("/tmp/composer-cache"),
                           "composer-cache".to_string()));

    ctx.environ.insert("COMPOSER_HOME".to_owned(),
                       "/usr/lib/composer".to_owned());
    ctx.environ.insert("COMPOSER_CACHE_DIR".to_owned(),
                       "/tmp/composer-cache".to_owned());

    Ok(())
}

pub fn setup_hhvm(distro: &mut Box<Distribution>, ctx: &mut Context)
    -> Result<(), StepError>
{
    let mut ubuntu = try!(distro.downcast_mut::<Ubuntu>().ok_or(
        StepError::UnsupportedFeatures(vec![packages::HHVM])
    ));

    let apt_key = AptKey {
        server: None,
        keys: vec![HHVM_APT_KEY.to_owned()],
    };

    try!(ubuntu.add_apt_key(ctx, &apt_key));
    let codename = try!(ubuntu::read_ubuntu_codename());

    let repo_info = UbuntuRepoInfo {
        url: HHVM_REPO_URL.to_owned(),
        suite: codename,
        components: vec!["main".to_owned()],
    };

    try!(ubuntu.add_debian_repo(ctx, &repo_info));
    try!(ubuntu.ensure_packages(ctx, &[packages::HHVM]));

    Ok(())
}

pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let mut cmd = try!(composer_cmd(ctx));
    cmd.arg("show");
    cmd.arg("-i");

    try!(capture_stdout(cmd)
        .and_then(|out| {
            File::create("/vagga/container/composer-list.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping package list: {}", e))
        }));

    Ok(())
}
