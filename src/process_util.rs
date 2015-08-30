use std::env;
use std::io::{Read};
use std::path::{Path, PathBuf};

use libc::getuid;
use unshare::{Command, Stdio, ExitStatus, UidMap, GidMap};

use container::uidmap::{Uidmap};
use path_util::PathExt;


pub static DEFAULT_PATH: &'static str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";


pub fn capture_stdout(mut cmd: Command) -> Result<Vec<u8>, String> {
    cmd.stdout(Stdio::piped());
    info!("Running {:?}", cmd);
    let mut child = try!(cmd.spawn()
        .map_err(|e| format!("{}", e)));
    let mut buf = Vec::with_capacity(1024);
    try!(child.stdout.take().unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e)));
    try!(child.wait().map_err(|e| format!("Error waiting for child: {}", e)));
    Ok(buf)
}

pub fn capture_stdout_status(mut cmd: Command)
    -> Result<(ExitStatus, Vec<u8>), String>
{
    cmd.stdout(Stdio::piped());
    info!("Running {:?}", cmd);
    let mut child = try!(cmd.spawn()
        .map_err(|e| format!("{}", e)));
    let mut buf = Vec::with_capacity(1024);
    try!(child.stdout.take().unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e)));
    let status = try!(child.wait()
        .map_err(|e| format!("Error waiting for child: {}", e)));
    Ok((status, buf))
}

pub fn convert_status(st: ExitStatus) -> i32 {
    match st {
        ExitStatus::Exited(c) => c as i32,
        ExitStatus::Signaled(s, _) => 128 + s,
    }
}

pub fn path_find<P: AsRef<Path>>(cmd: P, path: &str) -> Option<PathBuf> {
    let cmd = cmd.as_ref();
    trace!("Path search {:?} in {:?}", cmd, path);
    if cmd.is_absolute() {
        return Some(cmd.to_path_buf())
    }
    for prefix in path.split(":") {
        let tmppath = PathBuf::from(prefix).join(cmd);
        if tmppath.exists() {
            trace!("Path resolved {:?} is {:?}", cmd, tmppath);
            return Some(tmppath);
        }
    }
    None
}

pub fn env_path_find<P: AsRef<Path>>(cmd: P) -> Option<PathBuf> {
    env::var("PATH").map(|v| path_find(&cmd, &v[..]))
        .unwrap_or_else(|_| path_find(&cmd, DEFAULT_PATH))
}

pub fn set_uidmap(cmd: &mut Command, uid_map: &Uidmap, use_bin: bool) {
    match uid_map {
        &Uidmap::Singleton(uid, gid) => {
            cmd.set_id_maps(
                vec![UidMap { inside_uid: uid, outside_uid: 0, count: 1 }],
                vec![GidMap { inside_gid: gid, outside_gid: 0, count: 1 }]);
        }
        &Uidmap::Ranges(ref uids, ref gids) => {
            cmd.set_id_maps(
                uids.iter().map(|&(inu, outu, cntu)| UidMap {
                    inside_uid: inu, outside_uid: outu, count: cntu })
                    .collect(),
                gids.iter().map(|&(ing, outg, cntg)| GidMap {
                    inside_gid: ing, outside_gid: outg, count: cntg })
                    .collect(),
            );
            if use_bin && unsafe { getuid() } != 0 {
                let newuidmap = env_path_find("newuidmap")
                    .unwrap_or(PathBuf::from("/usr/bin/newuidmap"));
                let newgidmap = env_path_find("newgidmap")
                    .unwrap_or(PathBuf::from("/usr/bin/newgidmap"));
                cmd.set_id_map_commands(newuidmap, newgidmap);
            }
        }
    }
}
