use std::mem::transmute;
use std::collections::EnumSet;
use std::collections::enum_set::CLike;

use super::super::context::{BuildContext};
use super::generic::run_command;
use super::super::context as distr;
use super::debian;
use super::alpine;


#[repr(uint)]
#[deriving(Clone)]
pub enum PipFeatures {
    Dev,
    Pip,
    Git,
    Hg,
}

impl CLike for PipFeatures {
    fn to_uint(&self) -> uint {
        *self as uint
    }

    fn from_uint(v: uint) -> PipFeatures {
        unsafe { transmute(v) }
    }
}

#[deriving(Default)]
pub struct PipSettings {
    find_links: Vec<String>,
    fetch_deps: bool,
}


pub fn enable_deps(ctx: &mut BuildContext) {
    ctx.pip_settings.fetch_deps = true;
}

pub fn add_link(ctx: &mut BuildContext, lnk: &String) {
    ctx.pip_settings.find_links.push(lnk.to_string());
}

pub fn ensure_pip(ctx: &mut BuildContext, ver: u8,
    features: &EnumSet<PipFeatures>)
    -> Result<Path, String>
{
    match ctx.distribution {
        distr::Unknown => {
            return Err(format!("Unsupported distribution"));
        }
        distr::Ubuntu(_) => {
            return debian::ensure_pip(ctx, ver, features);
        }
        distr::Alpine(_) => {
            return alpine::ensure_pip(ctx, ver, features);
        }
    }
}

pub fn scan_features(pkgs: &Vec<String>) -> EnumSet<PipFeatures> {
    let mut res = EnumSet::empty();
    res.add(Dev);
    res.add(Pip);
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git+") {
            res.add(Git);
        } else if name.as_slice().starts_with("hg+") {
            res.add(Hg);
        }
    }
    return res;
}

pub fn pip_install(ctx: &mut BuildContext, ver: u8, pkgs: &Vec<String>)
    -> Result<(), String>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/pip-cache"),
                           "pip-cache".to_string()));
    ctx.environ.insert("PIP_DOWNLOAD_CACHE".to_string(),
                       "/tmp/pip-cache".to_string());
    let pip = try!(ensure_pip(ctx, ver, &scan_features(pkgs)));
    let mut args = vec!(
        pip.display().to_string(),  // Crappy, but but works in 99.99% cases
        "install".to_string(),
        );
    if !ctx.pip_settings.fetch_deps {
        args.push("--no-deps".to_string());
    }
    for lnk in ctx.pip_settings.find_links.iter() {
        args.push(format!("--find-links={}", lnk));
    }
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}
