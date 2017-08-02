use std::env;
use std::fmt;
use std::io::{self, Read};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::os::unix::io::FromRawFd;

use libc::{getuid, kill, c_int, pid_t};
use unshare::Signal::{SIGINT, SIGTERM, SIGCHLD, SIGTTIN, SIGTTOU, SIGCONT};
use unshare::Signal::{self, SIGQUIT, SIGTSTP, SIGSTOP};
use nix;
use nix::unistd::getpid;
use unshare;
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

lazy_static! {
    static ref DEBUG_STYLE: unshare::Style = unshare::Style::debug()
        .env(env::var("VAGGA_DEBUG_CMDENV")
             .map(|x| x.len() > 0).unwrap_or(false));
    // TODO(tailhook) Friendly style may turn into debug when some setting
    // is enabled
    static ref FRIENDLY_STYLE: unshare::Style = unshare::Style::short();
}


pub fn squash_stdio(cmd: &mut Command) -> Result<(), String> {
    let fd = nix::unistd::dup(2)
        .map_err(|e| format!("Can't duplicate fd 2: {}", e))?;
    cmd.stdout(Stdio::from_file(unsafe { File::from_raw_fd(fd) }));
    cmd.stdin(Stdio::null());
    Ok(())
}

pub fn capture_stdout(mut cmd: Command) -> Result<Vec<u8>, String> {
    cmd.stdout(Stdio::piped());
    info!("Running {}", cmd_show(&cmd));
    let mut child = cmd.spawn()
        .map_err(|e| format!("{}", e))?;
    let mut buf = Vec::with_capacity(1024);
    child.stdout.take().unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e))?;
    child.wait().map_err(|e| format!("Error waiting for child: {}", e))?;
    Ok(buf)
}

pub fn cmd_debug(cmd: &Command) -> unshare::Printer {
    cmd.display(&DEBUG_STYLE)
}

pub fn cmd_show(cmd: &Command) -> unshare::Printer {
    cmd.display(&FRIENDLY_STYLE)
}

pub fn cmd_err<E: fmt::Display>(cmd: &Command, err: E) -> String {
   format!("Error running {}: {}", cmd_debug(cmd), err)
}

pub fn capture_fd3(mut cmd: Command) -> Result<Vec<u8>, String>
{
    cmd.file_descriptor(3, Fd::piped_write());
    info!("Running {}", cmd_show(&cmd));
    let mut child = cmd.spawn()
        .map_err(|e| format!("Command {}: {}", cmd_debug(&cmd), e))?;
    let mut buf = Vec::with_capacity(1024);
    child.take_pipe_reader(3).unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e))?;
    let status = child.wait()
        .map_err(|e| format!("Error waiting for child: {}", e))?;
    if !status.success() {
        return Err(format!("Command {} {}", cmd_debug(&cmd), status));
    }
    Ok(buf)
}

pub fn capture_fd3_status(mut cmd: Command)
    -> Result<(ExitStatus, Vec<u8>), String>
{
    cmd.file_descriptor(3, Fd::piped_write());
    info!("Running {}", cmd_show(&cmd));
    let mut child = cmd.spawn()
        .map_err(|e| format!("{}", e))?;
    let mut buf = Vec::with_capacity(1024);
    child.take_pipe_reader(3).unwrap().read_to_end(&mut buf)
        .map_err(|e| format!("Error reading from pipe: {}", e))?;
    let status = child.wait()
        .map_err(|e| format!("Error waiting for child: {}", e))?;
    Ok((status, buf))
}

pub fn run_success(mut cmd: Command) -> Result<(), String> {
    debug!("Running {}", cmd_show(&cmd));
    match cmd.status() {
        Ok(ref st) if st.success() => Ok(()),
        Ok(status) => Err(cmd_err(&cmd, status)),
        Err(err) => Err(cmd_err(&cmd, err)),
    }
}

pub fn run_and_wait(cmd: &mut Command)
    -> Result<ExitStatus, String>
{
    // Trap must be installed before tty_guard because TTY guard relies on
    // SIGTTOU and SIGTTIN be masked out
    let mut trap = Trap::trap(&[SIGINT, SIGQUIT, SIGTERM, SIGCHLD,
                                SIGTTOU, SIGTTIN, SIGTSTP, SIGCONT]);

    let mut tty_guard = TtyGuard::new()
        .map_err(|e| format!("Error handling tty: {}", e))?;
    cmd.make_group_leader(true);

    info!("Running {}", cmd_show(&cmd));
    let child = cmd.spawn().map_err(|e| cmd_err(&cmd, e))?;
    let cmd_name = &format!("{:?}", cmd);
    let pid: i32 = getpid().into();

    for signal in trap.by_ref() {
        match signal {
            SIGINT|SIGQUIT|SIGCONT => {
                // SIGINT is usually a Ctrl+C, if we trap it here
                // child process hasn't controlling terminal,
                // so we send the signal to the child process
                debug!("Received {:?} signal. Propagating ..", signal);
                send_pg_signal(signal, child.pid(), &cmd_name);
            }
            SIGTSTP|SIGTTOU|SIGTTIN => {
                debug!("Received {:?} signal. Stopping child and self ..",
                    signal);
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
                            tty_guard.check().map_err(|e|
                                format!("Error handling tty: {}", e))?;
                            return Ok(status);
                        }
                        Stop(pid, SIGTTIN) | Stop(pid, SIGTTOU) => {
                            if let Err(e) = tty_guard.give(pid) {
                                // We shouldn't exit from here if we can't
                                // give a tty. Hopefull user will notice the
                                // error message.
                                // TODO(tailhook) may be kill the proccess?
                                error!("Can't give tty IO: {}", e);
                            } else if unsafe { killpg(pid, SIGCONT as i32) } == 1 {
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

pub fn send_signal(sig: Signal, pid: pid_t, cmd_name: &String) {
    if unsafe { kill(pid, sig as c_int) } < 0 {
        let e = io::Error::last_os_error();
        error!("Error sending {:?} to {:?}: {}", sig, cmd_name, e);
    }
}

pub fn send_pg_signal(sig: Signal, pid: pid_t, cmd_name: &String) {
    if unsafe { killpg(pid, sig as i32) } < 0 {
        let e = io::Error::last_os_error();
        error!("Error sending {:?} to {:?}: {}", sig, cmd_name, e);
    }
}

pub fn convert_status(st: ExitStatus) -> i32 {
    match st {
        ExitStatus::Exited(c) => c as i32,
        ExitStatus::Signaled(s, _) => 128 + s as i32,
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
    let gid_ranges = read_mapped_gids()?;
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
