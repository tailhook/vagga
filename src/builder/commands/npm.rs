use super::super::context::{BuildContext};
use super::generic::run_command;
use super::super::context as distr;
use super::super::dev;
use super::debian;
use super::alpine;

pub enum NpmFeatures {
    Dev,
    Npm,
    Rev(dev::RevControl),
}

pub fn scan_features(pkgs: &Vec<String>) -> Vec<NpmFeatures> {
    let mut res = vec!();
    res.push(Dev);
    res.push(Npm);
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git://") {
            res.push(Rev(dev::Git));
        }
    }
    return res;
}

pub fn ensure_npm(ctx: &mut BuildContext, features: &[NpmFeatures])
    -> Result<Path, String>
{
    match ctx.distribution {
        distr::Unknown => {
            //  Currently use alpine by default as it has smallest disk
            //  footprint
            try!(alpine::setup_base(ctx, &alpine::LATEST_VERSION.to_string()));
            return alpine::ensure_npm(ctx, features);
        }
        distr::Ubuntu(_) => {
            return debian::ensure_npm(ctx, features);
        }
        distr::Alpine(_) => {
            return alpine::ensure_npm(ctx, features);
        }
    }
}

pub fn npm_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    try!(ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string()));
    // TODO(tailhook) configure npm to use /tmp/npm-cache as cache dir
    //ctx.environ.insert("PIP_DOWNLOAD_CACHE".to_string(),
    //                   "/tmp/pip-cache".to_string());
    let npm = try!(ensure_npm(ctx, scan_features(pkgs).as_slice()));
    let mut args = vec!(
        npm.display().to_string(),  // Crappy, but but works in 99.99% cases
        "install".to_string(),
        "--user".to_string(), "root".to_string(),
        "--global".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}
