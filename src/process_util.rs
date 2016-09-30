use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::os::unix::io::FromRawFd;

use libc::{getuid, kill, c_int, pid_t};
use libc::{SIGINT, SIGTERM, SIGCHLD, SIGTTIN, SIGTTOU, SIGCONT};
use libc::{SIGQUIT, SIGTSTP, SIGSTOP};
use nix;
use nix::sys::signal::Signal;
use nix::unistd::getpid;
use unshare::{Command, Stdio, Fd, ExitStatus, UidMap, GidMap, child_events};
use signal::trap::Trap;

use config::Settings;
use container::uidmap::{Uidmap, read_mapped_gids};
use tty_util::{TtyGuard};


extern {
    pub fn killpg(pgrp: c_int, sig: c_int) -> c_int;
}

pub static DEFAULT_PATH: &'static str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

pub static PROXY_ENV_VARS: [&'static str; 5] =
    [ "http_proxy", "https_proxy", "ftp_proxy", "all_proxy", "no_proxy" ];


pub fn squash_stdio(cmd: &mut Command) -> Result<(), String> {
    let fd = try!(nix::unistd::dup(2)
        .map_err(|e| format!("Can't duplicate fd 2: {}", e)));
    cmd.stdout(unsafe { Stdio::from_raw_fd(fd) });
    cmd.stdin(Stdio::null());
    Ok(())
}

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

pub fn capture_fd3(mut cmd: Command) -> Result<Vec<u8>, String>
{
    cmd.file_descriptor(3, Fd::piped_write());
    info!("Running {:?}", cmd);
    let mut child = try!(cmd.spawn()
        .map_err(|e| format!("Command {:?}: {}", cmd, e)));
    let mut buf = Vec::with_capacity(1024);
    try!(child.take_pipe_reader(3).unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e)));
    let status = try!(child.wait()
        .map_err(|e| format!("Error waiting for child: {}", e)));
    if !status.success() {
        return Err(format!("Command {:?} {}", cmd, status));
    }
    Ok(buf)
}

pub fn capture_fd3_status(mut cmd: Command)
    -> Result<(ExitStatus, Vec<u8>), String>
{
    cmd.file_descriptor(3, Fd::piped_write());
    info!("Running {:?}", cmd);
    let mut child = try!(cmd.spawn()
        .map_err(|e| format!("{}", e)));
    let mut buf = Vec::with_capacity(1024);
    try!(child.take_pipe_reader(3).unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e)));
    let status = try!(child.wait()
        .map_err(|e| format!("Error waiting for child: {}", e)));
    Ok((status, buf))
}

pub fn run_and_wait(cmd: &mut Command)
    -> Result<ExitStatus, String>
{
    // Trap must be installed before tty_guard because TTY guard relies on
    // SIGTTOU and SIGTTIN be masked out
    let mut trap = Trap::trap(&[SIGINT, SIGQUIT, SIGTERM, SIGCHLD,
                                SIGTTOU, SIGTTIN, SIGTSTP, SIGCONT]);

    let mut tty_guard = try!(TtyGuard::new()
        .map_err(|e| format!("Error handling tty: {}", e)));
    cmd.make_group_leader(true);

    info!("Running {:?}", cmd);
    let child = try!(cmd.spawn()
                     .map_err(|e| format!("Error running {:?}: {}", cmd, e)));
    let cmd_name = &format!("{:?}", cmd);
    let pid = getpid();

    for signal in trap.by_ref() {
        match signal {
            SIGINT|SIGQUIT|SIGCONT => {
                // SIGINT is usually a Ctrl+C, if we trap it here
                // child process hasn't controlling terminal,
                // so we send the signal to the child process
                debug!("Received {:?} signal. Propagating ..",
                    get_sig_name(signal));
                send_pg_signal(signal, child.pid(), &cmd_name);
            }
            SIGTSTP|SIGTTOU|SIGTTIN => {
                debug!("Received {:?} signal. Stopping child and self ..",
                    get_sig_name(signal));
                send_pg_signal(SIGTSTP, child.pid(), &cmd_name);
                send_signal(SIGSTOP, pid, &pid.to_string());
            }
            SIGTERM => {
                debug!("Received SIGTERM signal. Propagating ..");
                send_signal(SIGTERM, child.pid(), &cmd_name);
            }
            SIGCHLD => {
                for event in child_events() {
                    use unshare::ChildEvent::*;
                    match event {
                        Death(pid, status) if pid == child.pid() => {
                            try!(tty_guard.check().map_err(|e|
                                format!("Error handling tty: {}", e)));
                            return Ok(status);
                        }
                        Stop(pid, SIGTTIN) | Stop(pid, SIGTTOU) => {
                            if let Err(e) = tty_guard.give(pid) {
                                // We shouldn't exit from here if we can't
                                // give a tty. Hopefull user will notice the
                                // error message.
                                // TODO(tailhook) may be kill the proccess?
                                error!("Can't give tty IO: {}", e);
                            } else if unsafe { killpg(pid, SIGCONT) } == 1 {
                                error!("Can't unpause pid {}: {}", pid,
                                    io::Error::last_os_error());
                            }
                        }
                        Stop(..) | Continue(..) | Death(..) => { }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    unreachable!();
}

pub fn send_signal(sig: c_int, pid: pid_t, cmd_name: &String) {
    if unsafe { kill(pid, sig) } < 0 {
        error!("Error sending {:?} to {:?}: {}",
            get_sig_name(sig), cmd_name, io::Error::last_os_error());
    }
}

pub fn send_pg_signal(sig: c_int, pid: pid_t, cmd_name: &String) {
    if unsafe { killpg(pid, sig) } < 0 {
        error!("Error sending {:?} to {:?}: {}",
            get_sig_name(sig), cmd_name, io::Error::last_os_error());
    }
}

pub fn get_sig_name(sig: c_int) -> String {
    Signal::from_c_int(sig).ok()
        .map_or(sig.to_string(), |s| format!("{:?}", s))
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

pub fn env_command<P: AsRef<Path>>(cmd: P) -> Command {
    if let Some(path) = env_path_find(cmd.as_ref()) {
        return Command::new(path);
    } else {
        // Even if we didn't find a command, we should fill default path and
        // let user inspect the failure when it happens, as more full command
        // description will be at that time
        return Command::new(cmd.as_ref());
    }
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
                let newuidmap = env_path_find("newuidmap");
                let newgidmap = env_path_find("newgidmap");
                if newuidmap.is_none() || newgidmap.is_none() {
                    warn!("Can't find `newuidmap` or `newgidmap` \
                        (see http://bit.ly/err_idmap)");
                }
                cmd.set_id_map_commands(
                    newuidmap.unwrap_or(PathBuf::from("/usr/bin/newuidmap")),
                    newgidmap.unwrap_or(PathBuf::from("/usr/bin/newgidmap")));
            }
        }
    }
}

pub fn set_fake_uidmap(cmd: &mut Command, uid: u32, external_uid:u32)
    -> Result<(), String>
{
    let gid_ranges = try!(read_mapped_gids());
    cmd.set_id_maps(vec![
            UidMap {
                inside_uid: uid,
                outside_uid: external_uid,
                count: 1
            }
        ],
        // Gid map is as always in this case
        gid_ranges.iter().map(|r| GidMap {
            inside_gid: r.start(), outside_gid: r.start(), count: r.len() })
            .collect(),
    );
    Ok(())
}

pub fn copy_env_vars(cmd: &mut Command, settings: &Settings) {
    cmd.env("TERM".to_string(),
            env::var_os("TERM").unwrap_or(From::from("dumb")));
    if settings.proxy_env_vars {
        for k in &PROXY_ENV_VARS {
            if let Some(v) = env::var_os(k) {
                cmd.env(k, v);
            }
        }
    }
}
