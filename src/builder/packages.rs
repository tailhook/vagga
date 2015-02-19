use super::context::Distribution as Distr;
use super::context::BuildContext;
use super::commands::debian;
use super::commands::alpine;

pub use self::Package::*;


// All packages should be installed as build dependency except specified
#[derive(Copy, Show, PartialEq, Eq)]
pub enum Package {
    BuildEssential,

    Python2,    // not build dep
    Python2Dev,
    Python3,    // not build dep
    Python3Dev,
    PipPy2,
    PipPy3,

    NodeJs,     // not build dep
    NodeJsDev,
    Npm,

    Git,
    Mercurial,
}


pub fn ensure_packages(ctx: &mut BuildContext, features: &[Package])
    -> Result<(), String>
{
    let left = match ctx.distribution {
        Distr::Unknown => {
            return Err(format!("Unsupported distribution"));
        }
        Distr::Ubuntu(_) => {
            try!(debian::ensure_packages(ctx, features))
        }
        Distr::Alpine(_) => {
            try!(alpine::ensure_packages(ctx, features))
        }
    };
    if left == vec!(PipPy2) {
        unimplemented!();
    } else if left == vec!(PipPy3) {
        unimplemented!();
    } else if left.len() > 0 {
        Err(format!("Features {:?} are not supported by distribution", left))
    } else {
        Ok(())
    }
}
