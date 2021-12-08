use builder::commands::composer;
use builder::commands::gem;
use builder::commands::npm;
use builder::commands::pip;
use builder::context::Context;
use builder::distrib::Distribution;
use builder::error::StepError;

pub use self::Package::*;

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

    Ruby,       // not build dep
    RubyDev,
    Bundler,

    NodeJs,     // not build dep
    NodeJsDev,
    Npm,
    Yarn,

    Php,        // not build dep
    PhpDev,
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
            PipPy2 => pip::bootstrap(ctx, 2)?,
            PipPy3 => pip::bootstrap(ctx, 3)?,
            Composer => composer::bootstrap(ctx)?,
            Bundler => gem::setup_bundler(ctx)?,
            Yarn => npm::setup_yarn(ctx)?,
            _ => {
                left.push(i);
                continue;
            }
        }
        ctx.featured_packages.insert(i);
    }
    return Ok(left);
}


pub fn ensure_packages(distro: &mut Box<dyn Distribution>, ctx: &mut Context,
    features: &[Package])
    -> Result<(), StepError>
{
    let mut features = features.iter().cloned()
        .filter(|x| !ctx.featured_packages.contains(x))
        .collect::<Vec<Package>>();
    if features.len() > 0 {
        let leftover = distro.ensure_packages(ctx, &features)?;
        ctx.featured_packages.extend(
            features.into_iter().filter(|x| !leftover.contains(x)));
        features = leftover;
    }
    if features.len() > 0 {
        features = generic_packages(ctx, features)?;
    }
    if features.len() > 0 {
        return Err(StepError::UnsupportedFeatures(features));
    } else {
        Ok(())
    }
}
