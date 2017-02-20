use std::env;
use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};
use unshare::{Namespace};

use capsule::Context;
use launcher::wrap::Wrapper;
use options::build_mode::BuildMode;
use process_util::{capture_fd3, copy_env_vars, squash_stdio};

pub fn run_command(context: &Context, args: Vec<String>)
    -> Result<i32, String>
{
    unimplemented!();
}
