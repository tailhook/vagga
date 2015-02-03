use super::super::context::{BuildContext};
use super::generic::run_command;
use super::super::context::Distribution as Distr;
use super::super::dev::RevControl;
use super::debian;
use super::alpine;
use self::PipFeatures::*;


pub enum PipFeatures {
    Dev,
    Pip,
    Rev(RevControl),
}


pub fn ensure_pip(ctx: &mut BuildContext, ver: u8,
    features: &[PipFeatures])
    -> Result<Path, String>
{
    match ctx.distribution {
        Distr::Unknown => {
            return Err(format!("Unsupported distribution"));
        }
        Distr::Ubuntu(_) => {
            return debian::ensure_pip(ctx, ver, features);
        }
        Distr::Alpine(_) => {
            return alpine::ensure_pip(ctx, ver, features);
        }
    }
}

pub fn scan_features(pkgs: &Vec<String>) -> Vec<PipFeatures> {
    let mut res = vec!();
    res.push(Dev);
    res.push(Pip);
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git+") {
            res.push(Rev(RevControl::Git));
        } else if name.as_slice().starts_with("hg+") {
            res.push(Rev(RevControl::Hg));
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
    let pip = try!(ensure_pip(ctx, ver, scan_features(pkgs).as_slice()));
    let mut args = vec!(
        pip.display().to_string(),  // Crappy, but but works in 99.99% cases
        "install".to_string(),
        );
    if ctx.pip_settings.index_urls.len() > 0 {
        let mut indexes = ctx.pip_settings.index_urls.iter();
        if let Some(ref lnk) = indexes.next() {
            args.push(format!("--index-url={}", lnk));
            for lnk in indexes {
                args.push(format!("--extra-index-url={}", lnk));
            }
        }
    }
    if !ctx.pip_settings.dependencies {
        args.push("--no-deps".to_string());
    }
    for lnk in ctx.pip_settings.find_links.iter() {
        args.push(format!("--find-links={}", lnk));
    }
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}
