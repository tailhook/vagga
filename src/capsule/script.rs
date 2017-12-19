use std::io::{stdout, stderr};
use std::sync::Arc;

use argparse::{ArgumentParser};
use argparse::{List, Store, StoreOption, StoreTrue};
use unshare::{Command};

use capsule::Context;
use capsule::packages::State;
use capsule::download;
use launcher::wrap::Wrapper;
use process_util::{run_and_wait, convert_status};


pub fn run_script(context: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut url = "".to_string();
    let mut sha256 = None;
    let mut refresh = false;
    {
        args.insert(0, "vagga _capsule script".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Downloads script if not cached, puts it into a cache and starts.
            ");
        ap.refer(&mut sha256)
            .add_option(&["--sha256"], StoreOption,
                "A SHA256 hashsum of a script (if you want to check)");
        ap.refer(&mut refresh)
            .add_option(&["--refresh"], StoreTrue,
                "Download file even if there is a cached item");
        ap.refer(&mut url)
            .add_argument("url", Store,
                "A script to run")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arg", List, "Arguments to the script");
        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    // TODO(tailhook) wrap settings into Arc in the launcher's main
    let mut capsule = State::new(&Arc::new(context.settings.clone()));
    let (path, _) = download::maybe_download_and_check_hashsum(
        &mut capsule, &url, sha256, refresh)?;

    let mut cmd: Command = Command::new("/bin/sh");
    cmd.workdir(&context.workdir);
    cmd.arg(&path);
    cmd.args(&cmdargs);
    let res = run_and_wait(&mut cmd).map(convert_status);

    return res;
}
