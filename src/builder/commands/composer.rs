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
use builder::download;
use config::builders::{ComposerSettings, ComposerReqInfo};
use process_util::capture_stdout;
use file_util::{copy, create_dir};

const DEFAULT_RUNTIME: &'static str = "/usr/bin/php";
const COMPOSER_HOME: &'static str = "/usr/local/lib/composer";
const COMPOSER_BOOTSTRAP: &'static str = "https://getcomposer.org/installer";


impl Default for ComposerSettings {
    fn default() -> Self {
        ComposerSettings {
            install_runtime: true,
            runtime_exe: None,
        }
    }
}

fn scan_features(settings: &ComposerSettings, _prefer_dist: bool)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);
    res.push(packages::Composer);
    if settings.install_runtime {
        res.push(packages::Php);
        res.push(packages::PhpDev)
    }
    // Probably it's not it worth trying to figure out whether we need Git or Mercurial and it is
    // more likely that a php project is using Git, therefore it is reasonable to simply assume we
    // always need Git
    res.push(packages::Git);
    return res;
}

fn composer_cmd(ctx: &mut Context) -> Result<Command, StepError> {
    let runtime = ctx.composer_settings
        .runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());
    let mut cmd = try!(command(ctx, runtime));
    cmd.arg("/tmp/composer.phar");
    Ok(cmd)
}

pub fn composer_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.composer_settings, true);

    try!(packages::ensure_packages(distro, ctx, &features));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = try!(composer_cmd(ctx));
    cmd.args(&["global", "require", "--prefer-dist"]);
    cmd.args(pkgs);
    try!(run(cmd));

    // link package binaries into /usr/local/bin to be available in PATH
    let args = vec!(
        "/bin/ln".to_owned(),
        "-s".to_owned(),
        "/usr/lib/composer/vendor/bin/*".to_owned(),
        "/usr/local/bin/".to_owned());
    run_command(ctx, &args)
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
    ctx.add_ensure_dir(Path::new("/usr/lib/composer/vendor"));
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

pub fn bootstrap(ctx: &mut Context) -> Result<(), String> {
    try_msg!(create_dir(COMPOSER_HOME, true),
         "Error creating composer home dir {d:?}: {err}", d=COMPOSER_HOME);

    let composer_inst = try!(download::download_file(ctx, COMPOSER_BOOTSTRAP));
    try!(copy(&composer_inst, &Path::new("/vagga/root/tmp/composer-setup.php"))
        .map_err(|e| format!("Error copying composer installer: {}", e)));

    let runtime_exe = ctx.composer_settings
        .runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());

    let args = [
        runtime_exe,
        "/tmp/composer-setup.php".to_owned(),
        "--install-dir=/tmp/".to_owned(),
    ];

    run_command(ctx, &args)
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
