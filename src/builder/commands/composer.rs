use std::path::Path;
use std::fs::File;
use std::io::Write;

use super::super::context::{Context};
use super::super::packages;
use super::generic::{run_command, capture_command};
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use config::builders::{ComposerSettings, ComposerReqInfo};

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

fn composer_engine(ctx: &mut Context) -> String {
    if let Some(ref exe) = ctx.composer_settings.engine_exe {
        exe.to_owned()
    } else if ctx.composer_settings.engine == "php" {
        "php".to_owned()
    } else if ctx.composer_settings.engine == "hhvm" {
        "hhvm".to_owned()
    } else {
        unreachable!();
    }
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

    let engine_exe = composer_engine(ctx);
    let mut cmd = try!(command(ctx, &engine_exe));
    cmd.arg("/tmp/composer.phar");
    cmd.arg("global");
    cmd.arg("require");
    cmd.arg("--prefer-dist");
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

    let engine_exe = composer_engine(ctx);
    let mut cmd = try!(command(ctx, &engine_exe));
    cmd.arg("/tmp/composer.phar");

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
    if ctx.binary_ident.contains("ubuntu") {
        ctx.composer_settings.engine_exe = Some("php5".to_owned());
    }

    ctx.add_ensure_dir(Path::new("/tmp/composer/vendor"));
    let args = vec!(
        "ln".to_owned(),
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

pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let engine_exe = composer_engine(ctx);
    let args = vec!(
        engine_exe,
        "/tmp/composer.phar".to_owned(),
        "show".to_string(),
        "i".to_string(),
    );
    try!(capture_command(ctx, &args, &[])
        .and_then(|out| {
            File::create("/vagga/container/composer-list.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping package list: {}", e))
        }));

    Ok(())
}
