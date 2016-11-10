/// The "capsule" module is a module handling alpine linux distribution that
/// is used in a build shell.
///
/// Usually we only use busybox from capsule to download initial image, but we
/// may need real wget and ca-certificates for https. An other features may
/// need more things.

use std::collections::HashSet;
use std::fs::{File};
use std::io::{Write};
use std::path::Path;

use unshare::Command;
use libmount::BindMount;

use config::settings::Settings;
use process_util::squash_stdio;
use super::context::Context;
use super::commands::alpine::{LATEST_VERSION, choose_mirror};
use super::super::file_util::Dir;

pub use self::Feature::*;


#[derive(Clone, Copy)]
pub enum Feature {
    Https,
    Gzip,
    Bzip2,
    Xz,
    AlpineInstaller,
    Git,
}

#[derive(Default)]
pub struct State {
    capsule_base: bool,
    //alpine_ready: bool,
    installed_packages: HashSet<String>,
}

// Also used in alpine
pub fn apk_run(args: &[&str], packages: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("/vagga/bin/apk");
    squash_stdio(&mut cmd)?;
    cmd.env("PATH", "/vagga/bin")
        .args(args)
        .args(packages);
    debug!("Running APK {:?}", cmd);
    return match cmd.status().map_err(|e| format!("Can't run apk: {}", e))
    {
        Ok(s) if s.success() => Ok(()),
        Ok(val) => Err(format!("Apk exited with status: {}", val)),
        Err(x) => Err(format!("Error running apk: {}", x)),
    }
}


pub fn ensure_features(ctx: &mut Context, features: &[Feature])
    -> Result<(), String>
{
    return ensure(&mut ctx.capsule, &ctx.settings, features);
}

pub fn ensure(capsule: &mut State, settings: &Settings, features: &[Feature])
    -> Result<(), String>
{
    if features.len() == 0 {
        return Ok(());
    }
    if !capsule.capsule_base {
        let cache_dir = Path::new("/vagga/cache/alpine-cache");
        if !cache_dir.exists() {
            try_msg!(Dir::new(&cache_dir).create(),
                 "Error creating cache dir: {err}");
        }
        let path = Path::new("/etc/apk/cache");
        try_msg!(Dir::new(&path).recursive(true).create(),
             "Error creating cache dir: {err}");
        BindMount::new(&cache_dir, &path).mount()
             .map_err(|e| e.to_string())?;

        apk_run(&[
            "--allow-untrusted",
            "--initdb",
            "add",
            "--force",
            "/vagga/bin/alpine-keys.apk",
            ], &[])?;
        let mirror = settings.alpine_mirror.clone()
            .unwrap_or(choose_mirror());
        File::create(&Path::new("/etc/apk/repositories"))
            .and_then(|mut f| write!(&mut f, "{}{}/main\n",
                mirror, LATEST_VERSION))
            .map_err(|e| format!("Can't write repositories file: {}", e))?;
        capsule.capsule_base = true;
    }
    let mut pkg_queue = vec!();
    for value in features.iter() {
        match *value {
            AlpineInstaller => {}  // basically capsule_base
            Https => {
                pkg_queue.push("wget".to_string());
                pkg_queue.push("ca-certificates".to_string());
            }
            Gzip => {
                pkg_queue.push("gzip".to_string());
            }
            Bzip2 => {
                pkg_queue.push("bzip2".to_string());
            }
            Xz => {
                pkg_queue.push("xz".to_string());
            }
            Git => {
                pkg_queue.push("git".to_string());
                pkg_queue.push("ca-certificates".to_string());
            }
        }
    }
    if pkg_queue.len() > 0 {
        pkg_queue = pkg_queue.into_iter()
            .filter(|x| !capsule.installed_packages.contains(x))
            .collect();
    }
    if pkg_queue.len() > 0 {
        if capsule.installed_packages.len() == 0 { // already have indexes
            apk_run(&[
                "--update-cache",
                "add",
                ], &pkg_queue[0..])?;
        } else {
            apk_run(&[
                "add",
                ], &pkg_queue[0..])?;
        }
        capsule.installed_packages.extend(pkg_queue.into_iter());
    }
    Ok(())
}

