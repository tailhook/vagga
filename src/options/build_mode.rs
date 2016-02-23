use std::default::Default;

use argparse::{ArgumentParser, StoreConst};


#[derive(Clone, Copy, Debug)]
pub enum BuildMode {
    Normal,
    NoImage,
    NoBuild,
    NoVersion,
}

impl Default for BuildMode {
    fn default() -> BuildMode {
        BuildMode::Normal
    }
}

pub fn build_mode<'x>(ap: &mut ArgumentParser<'x>, mode: &'x mut BuildMode)
{
    ap.refer(mode)
    .add_option(&["--no-image"], StoreConst(BuildMode::NoImage), "
        Do not download container image from image index.
        ")
    .add_option(&["--no-build"], StoreConst(BuildMode::NoBuild), "
        Do not build container even if it is out of date. Return error
        code 29 if it's out of date.")
    .add_option(&["--no-version-check"], StoreConst(BuildMode::NoVersion), "
        Do not run versioning code, just pick whatever container
        version with the name was run last (or actually whatever is
        symlinked under `.vagga/container_name`). Implies `--no-build`
        ");
}
