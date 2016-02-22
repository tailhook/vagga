use std::path::Path;

use super::context::Context;
use super::commands::generic::run_command_at_env;
use super::download;
use builder::error::StepError;
use builder::distrib::Distribution;
use file_util::{copy, create_dir};

pub use self::Package::*;

const COMPOSER_BOOTSTRAP: &'static str = "https://getcomposer.org/installer";


// All packages should be installed as build dependency except specified
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Package {
    BuildEssential,
    Https,

    Python2,    // not build dep
    Python2Dev,
    Python3,    // not build dep
    Python3Dev,
    PipPy2,
    PipPy3,

    NodeJs,     // not build dep
    NodeJsDev,
    Npm,

    PHP,
    HHVM,
    Composer,

    Git,
    Mercurial,

}


fn generic_packages(ctx: &mut Context, features: Vec<Package>)
    -> Result<Vec<Package>, String>
{
    let mut left = vec!();
    for i in features.into_iter() {
        match i {
            PipPy2 | PipPy3 => {
                let args = vec!(
                    ctx.pip_settings.python_exe.clone()
                    .unwrap_or((if i == PipPy2 {"python2"} else {"python3"})
                               .to_string()),
                    "/tmp/get-pip.py".to_string(),
                    "--target=/tmp/pip-install".to_string(),
                    );
                let pip_inst = try!(download::download_file(ctx,
                    "https://bootstrap.pypa.io/get-pip.py"));
                try!(copy(&pip_inst, &Path::new("/vagga/root/tmp/get-pip.py"))
                    .map_err(|e| format!("Error copying pip: {}", e)));
                try!(run_command_at_env(ctx, &args, &Path::new("/work"), &[]));
            }
            Composer => {
                let composer_home = Path::new("/vagga/root/tmp/composer");
                try_msg!(create_dir(&composer_home, true),
                     "Error creating composer home dir {d:?}: {err}", d=composer_home);

                let composer_inst = try!(download::download_file(ctx, COMPOSER_BOOTSTRAP));
                try!(copy(&composer_inst, &Path::new("/vagga/root/tmp/composer-setup.php"))
                    .map_err(|e| format!("Error copying composer: {}", e)));

                let engine_exe = if ctx.composer_settings.engine == "php" {
                    if ctx.binary_ident.contains("ubuntu") {
                        "php5"
                    } else {
                        "php"
                    }
                } else {
                    "hhvm"
                };

                let args = vec!(
                    engine_exe.to_owned(),
                    "/tmp/composer-setup.php".to_owned(),
                    "--install-dir=/tmp/composer/".to_owned(),
                    );

                try!(run_command_at_env(ctx, &args, &Path::new("/work"), &[]));
            }
            _ => {
                left.push(i);
                continue;
            }
        }
        ctx.featured_packages.insert(i);
    }
    return Ok(left);
}


pub fn ensure_packages(distro: &mut Box<Distribution>, ctx: &mut Context,
    features: &[Package])
    -> Result<(), StepError>
{
    let mut features = features.iter().cloned()
        .filter(|x| !ctx.featured_packages.contains(x))
        .collect::<Vec<Package>>();
    if features.len() > 0 {
        let leftover = try!(distro.ensure_packages(ctx, &features));
        ctx.featured_packages.extend(
            features.into_iter().filter(|x| !leftover.contains(x)));
        features = leftover;
    }
    if features.len() > 0 {
        features = try!(generic_packages(ctx, features));
    }
    if features.len() > 0 {
        return Err(StepError::UnsupportedFeatures(features));
    } else {
        Ok(())
    }
}
