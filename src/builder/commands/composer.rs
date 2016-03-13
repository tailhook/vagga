use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs as unix_fs;

use unshare::{Command};
use scan_dir::{self, ScanDir};
use regex::Regex;

use super::super::context::{Context};
use super::super::packages;
use super::generic::{run_command, capture_command};
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use builder::download;
use config::builders::{ComposerSettings, ComposerDepInfo};
use process_util::capture_stdout;
use file_util;

const DEFAULT_RUNTIME: &'static str = "/usr/bin/php";
const DEFAULT_INCLUDE_PATH: &'static str = ".:/usr/local/lib/composer";

const COMPOSER_HOME: &'static str = "/usr/local/lib/composer";
const COMPOSER_CACHE: &'static str = "/tmp/composer-cache";
const COMPOSER_VENDOR_DIR: &'static str = "/usr/local/lib/composer/vendor";
const COMPOSER_BIN_DIR: &'static str = "/usr/local/bin";
const COMPOSER_BOOTSTRAP: &'static str = "https://getcomposer.org/installer";


impl Default for ComposerSettings {
    fn default() -> Self {
        ComposerSettings {
            install_runtime: true,
            install_dev: false,
            runtime_exe: None,
            include_path: None,
        }
    }
}

fn scan_features(settings: &ComposerSettings)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::Https);
    res.push(packages::Composer);
    if settings.install_runtime {
        res.push(packages::Php);
        if settings.install_dev {
            res.push(packages::BuildEssential);
            res.push(packages::PhpDev)
        }
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
    cmd.arg("--no-interaction");
    Ok(cmd)
}

pub fn composer_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.composer_settings);
    try!(packages::ensure_packages(distro, ctx, &features));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = try!(composer_cmd(ctx));
    cmd.args(&["global", "require", "--prefer-dist", "--update-no-dev"]);
    cmd.args(pkgs);
    try!(run(cmd));
    Ok(())
}

pub fn composer_dependencies(distro: &mut Box<Distribution>,
    ctx: &mut Context, info: &ComposerDepInfo)
    -> Result<(), StepError>
{
    let features = scan_features(&ctx.composer_settings);
    try!(packages::ensure_packages(distro, ctx, &features));

    let mut cmd = try!(composer_cmd(ctx));
    cmd.arg("install");
    if let Some(ref dir) = info.working_dir {
        cmd.arg(format!("--working-dir={}", dir));
    }
    if !info.dev { cmd.arg("--no-dev"); }
    if info.ignore_platform_reqs { cmd.arg("--ignore-platform-reqs"); }
    if info.no_autoloader { cmd.arg("--no_autoloader"); }
    if info.no_scripts { cmd.arg("--no-scripts"); }
    if info.no_plugins { cmd.arg("--no-plugins"); }
    if info.optimize_autoloader { cmd.arg("--optimize-autoloader"); }

    match info.prefer {
        Some(ref p) if p == "dist" => { cmd.arg("--prefer-dist"); },
        Some(ref p) if p == "source" => { cmd.arg("--prefer-source"); },
        Some(ref p) => return Err(From::from(format!(
            "Value of 'ComposerDependencies.prefer' must be either \
            'source' or 'dist', '{}' given", p
        ))),
        _ => {}
    }

    run(cmd)
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
    try!(ctx.add_cache_dir(Path::new("/tmp/composer-cache"),
                           "composer-cache".to_string()));

    ctx.environ.insert("COMPOSER_HOME".to_owned(),
                       COMPOSER_HOME.to_owned());
    ctx.environ.insert("COMPOSER_VENDOR_DIR".to_owned(),
                       COMPOSER_VENDOR_DIR.to_owned());
    ctx.environ.insert("COMPOSER_BIN_DIR".to_owned(),
                       COMPOSER_BIN_DIR.to_owned());
    ctx.environ.insert("COMPOSER_CACHE_DIR".to_owned(),
                       COMPOSER_CACHE.to_owned());

    Ok(())
}

pub fn bootstrap(ctx: &mut Context) -> Result<(), String> {
    try_msg!(file_util::create_dir(COMPOSER_HOME, true),
        "Error creating composer home dir {d:?}: {err}", d=COMPOSER_HOME);

    let composer_inst = try!(download::download_file(ctx, COMPOSER_BOOTSTRAP));
    try!(file_util::copy(&composer_inst, &Path::new("/vagga/root/tmp/composer-setup.php"))
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
    try!(run_command(ctx, &args));

    try_msg!(unix_fs::symlink("/vagga/root/tmp/composer.phar", "/vagga/root/usr/local/bin/composer"),
        "Error creating symlink '/usr/local/bin/composer -> /tmp/composer.phar': {err}");

    if ctx.composer_settings.install_runtime {
        try!(setup_include_path(ctx));
    }

    Ok(())
}

fn setup_include_path(ctx: &mut Context) -> Result<(), String> {
    let vagga_ini_content = {
        let include_path = ctx.composer_settings
            .include_path.clone()
            .unwrap_or(DEFAULT_INCLUDE_PATH.to_owned());
        format!("include_path={}", include_path)
    };

    let conf_dirs = try_msg!(find_conf_dirs(),
                             "Error listing PHP configuration directories: {err}");

    for conf_d in conf_dirs.iter() {
        // create vagga.ini file
        try!(create_vagga_ini(&conf_d.join("vagga.ini"), &vagga_ini_content));
    }

    if !conf_dirs.is_empty() {
        // vagga.ini file(s) created, we're done here
        return Ok(())
    }

    // If we didn't find any conf.d, ask 'php --ini'
    let conf_d = try!(ask_php_for_conf_d(ctx));

    // create conf.d
    if !conf_d.exists() {
        try!(file_util::create_dir(&conf_d, true)
        .map_err(|e| format!("Error creating directory {:?}: {}", conf_d, e)));
    }
    // create vagga.ini file
    try!(create_vagga_ini(&conf_d, &vagga_ini_content));

    Ok(())
}

fn create_vagga_ini(location: &Path, content: &str) -> Result<(), String> {
    File::create(location)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| format!("Error creating file {:?}: {}", location, e))
}

fn find_conf_dirs() -> Result<Vec<PathBuf>, scan_dir::Error> {
    ScanDir::dirs().skip_symlinks(true).read("/vagga/root/etc", |iter| {
        iter.filter(|&(_, ref name)| name.starts_with("php"))
        .flat_map(|(ref entry, _)| {
            ScanDir::dirs().read(entry.path(), |iter| {
                iter.filter(|&(ref entry, ref name)| {
                    name == "conf.d" ||
                    entry.path().join("conf.d").exists()
                })
                .map(|(ref entry, _)| {
                    let path = entry.path();
                    if path.ends_with("conf.d") { path }
                    else { path.join("conf.d") }
                })
                .collect()
            })
        })
        .collect()
    })
}

fn ask_php_for_conf_d(ctx: &mut Context) -> Result<PathBuf, String> {
    let runtime_exe = ctx.composer_settings
        .runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());

    let args = [runtime_exe, "--ini".to_owned()];
    let output = try!(capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error parsing command output: {}", e)))
        .map_err(|e| format!("Error reading command output: {}", e)));

    // match any line that ends with /etc/php*/**/conf.d, get first result
    let re = Regex::new(r#"(?m).*?(/etc/php\d/.*?conf.d)$"#).expect("Invalid regex");

    let conf_d = try!(re.captures(&output)
        .and_then(|cap| cap.at(1))
        .ok_or("PHP configuration directory was not found".to_owned()));

    Ok(PathBuf::from(conf_d))
}

pub fn finish(ctx: &mut Context) -> Result<(), StepError> {
    try!(list_packages(ctx));
    try!(fs::remove_file(Path::new("/vagga/root/usr/local/bin/composer"))
        .map_err(|e| format!("Error removing symlink '/usr/local/bin/composer': {}", e)));

    Ok(())
}

fn list_packages(ctx: &mut Context) -> Result<(), StepError> {
    let mut cmd = try!(composer_cmd(ctx));
    cmd.arg("show");

    try!(capture_stdout(cmd)
        .and_then(|out| {
            File::create("/vagga/container/composer-list.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping composer package list: {}", e))
        }));

    Ok(())
}
