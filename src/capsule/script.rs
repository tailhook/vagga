use std::io::{stdout, stderr};

use argparse::{ArgumentParser};
use argparse::{List, Store};
use unshare::{Command};

use capsule::Context;
use launcher::wrap::Wrapper;
use process_util::{run_and_wait, convert_status};


pub fn run_script(context: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut url = "".to_string();
    {
        args.insert(0, "vagga _capsule script".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Downloads script if not cached, puts it into a cache and starts.
            ");
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
    let path = if url.starts_with("http://") || url.starts_with("https://") {
        unimplemented!();
    } else if url.starts_with('.') {
        url
    } else {
        error!("Wrong url {:?}. Url must start \
            either with a `http://` or `https://` or with a dot `./something`\
            for local paths", url);
        return Ok(122);
    };

    let mut cmd: Command = Command::new("/bin/sh");
    cmd.workdir(&context.workdir);
    cmd.arg(&path);
    cmd.args(&cmdargs);
    let res = run_and_wait(&mut cmd).map(convert_status);

    return res;
}
