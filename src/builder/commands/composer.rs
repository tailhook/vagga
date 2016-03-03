use std::path::Path;
use std::fs::File;
use std::io::Write;

use unshare::{Command};

use super::super::context::{Context};
use super::super::packages;
use super::generic::{run_command, capture_command};
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use builder::download;
use config::builders::{ComposerSettings, ComposerDepInfo};
use process_util::capture_stdout;
use file_util::{copy, create_dir};

const DEFAULT_RUNTIME: &'static str = "/usr/bin/php";
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
    try!(run_command(ctx, &args));

    let args = [
        "ln".to_owned(),
        "-s".to_owned(),
        "/tmp/composer.phar".to_owned(),
        "/usr/local/bin/composer".to_owned(),
    ];
    try!(run_command(ctx, &args));

    if ctx.composer_settings.install_runtime {
        try!(setup_include_path(ctx));
    }

    Ok(())
}

fn setup_include_path(ctx: &mut Context) -> Result<(), String> {
    let args = [
        "/bin/sh".to_owned(),
        "-exc".to_owned(),
        "find $(ls -d /etc/php*) -name 'conf.d' | grep -E '^/etc/php'".to_owned(),
    ];

    let conf_d = try!(capture_command(ctx, &args, &[])
        .and_then(|result| {
            String::from_utf8(result).map_err(|e| format!("{}", e))
        }));

    let conf_d: Vec<String> = {
        let conf_d_lines: Vec<String> = conf_d
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_owned())
            .collect();

        if !conf_d_lines.is_empty() {
            conf_d_lines
        } else {
            let args = [
                "/bin/sh".to_owned(),
                "-exc".to_owned(),
                "php --ini | grep -E 'conf.d$' | cut -d ':' -f 2 | cut -d ' ' -f 2".to_owned()
            ];
            let conf = try!(capture_command(ctx, &args, &[])
                .and_then(|result| {
                    String::from_utf8(result)
                    .map_err(|e| format!("{}", e))
                }));
            let args = [
                "mkdir".to_owned(),
                "-p".to_owned(),
                conf.clone(),
            ];
            try!(run_command(ctx, &args));
            vec!(conf)
        }
    };

    for conf in conf_d {
        let args = [
            "/bin/sh".to_owned(),
            "-exc".to_owned(),
            format!("echo 'include_path=.:/usr/local/lib/composer' > {}/vagga.ini", &conf),
        ];
        try!(run_command(ctx, &args));
    }

    Ok(())
}

pub fn finish(ctx: &mut Context) -> Result<(), StepError> {
    try!(list(ctx));
    try!(run_command(ctx, &[
        "rm".to_owned(),
        "/usr/local/bin/composer".to_owned(),
    ]));
    Ok(())
}

fn list(ctx: &mut Context) -> Result<(), StepError> {
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
