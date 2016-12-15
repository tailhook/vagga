use std::path::PathBuf;
use std::process::exit;
use std::io::{stdout, stderr, Write};

use argparse::{ArgumentParser, Store, StoreOption, List};
use unshare::Command;

use process_util::{set_fake_uidmap, cmd_err};

fn run_as(args: Vec<String>) -> Result<i32, String> {
    let mut user_id: Option<u32> = None;
    let mut group_id: Option<u32> = None;
    let mut supplementary_gids: Vec<u32> = vec!();
    let mut external_user_id: Option<u32> = None;
    let mut work_dir = "/work".to_string();
    let mut script = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Run as.");
        ap.refer(&mut user_id)
            .add_option(&["--user-id"], StoreOption,
                "User id");
        ap.refer(&mut group_id)
            .add_option(&["--group-id"], StoreOption,
                "Group id");
        ap.refer(&mut supplementary_gids)
            .add_option(&["--supplementary-gids"], List,
                "Supplementary gids");
        ap.refer(&mut work_dir)
            .add_option(&["--work-dir"], Store,
                "Work directory");
        ap.refer(&mut external_user_id)
            .add_option(&["--external-user-id"], StoreOption,
                "External user id");
        ap.refer(&mut script)
            .add_argument("script", Store,
                "Script")
            .required();
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {},
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }

    let mut cmd = Command::new("/bin/sh");
    cmd.chroot_dir("/vagga/root");
    cmd.arg("-exc");
    cmd.arg(&script);
    cmd.current_dir(&PathBuf::from(work_dir));

    let uid = user_id.unwrap_or(0);
    let gid = group_id.unwrap_or(0);
    if let Some(euid) = external_user_id {
        try_msg!(set_fake_uidmap(&mut cmd, uid, euid), "Cannot set uidmap: {err}");
    }
    cmd.uid(uid);
    cmd.gid(gid);
    cmd.groups(supplementary_gids.clone());

    match cmd.status() {
        Ok(st) if st.success() => Ok(0),
        Ok(s) => Err(cmd_err(&cmd, s)),
        Err(e) => Err(cmd_err(&cmd, e)),
    }
}

fn run() -> i32 {
    let mut cmd = "".to_string();
    let mut args: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs some specific commands that needs intermediate process.
            ");
        ap.refer(&mut cmd)
            .add_argument("cmd", Store,
                "Command");
        ap.refer(&mut args)
            .add_argument("options", List,
                "Options specific for this command");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }
    args.insert(0, format!("vagga_runner {}", cmd));
    let res: Result<i32, String> = match &cmd[..] {
        "run_as" => run_as(args),
        _ => Err(format!("Unknown subcommand: {}", cmd))
    };

    match res {
        Ok(rc) => return rc,
        Err(text) => {
            writeln!(&mut stderr(), "{}", text).ok();
            return 121;
        }
    }
}

pub fn main() {
    let val = run();
    exit(val);
}
