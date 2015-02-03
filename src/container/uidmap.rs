use std::os::getenv;
use std::io::{IoError, OtherIoError};
use std::io::{File, Open, Write};
use std::io::{BufferedReader, MemWriter};
use std::cmp::min;
use std::cmp::Ordering;
use std::str::FromStr;
use std::str::from_utf8;
use std::io::process::{ExitStatus, ExitSignal, Command, Ignored, InheritFd};

use libc::funcs::posix88::unistd::{geteuid, getegid};
use libc::{pid_t, uid_t, gid_t};

use config::Range;
use config::Settings;
use self::Uidmap::*;

#[derive(Clone)]
pub enum Uidmap {
    Singleton(uid_t, gid_t),
    Ranges(Vec<(uid_t, uid_t, uid_t)>, Vec<(gid_t, gid_t, gid_t)>),
}


fn read_uid_map(username: &str) -> Result<Vec<Range>,String> {
    let file = File::open(&Path::new("/etc/subuid"));
    let mut res = Vec::new();
    let mut reader = BufferedReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try!(line.map_err(
            |e| format!("Error reading /etc/subuid: {}", e)));
        let parts: Vec<&str> = line.as_slice().split(':').collect();
        let start = FromStr::from_str(parts[1]);
        let count = FromStr::from_str(parts[2].trim_right());
        if parts.len() != 3 || start.is_none() || count.is_none() {
            return Err(format!("/etc/subuid:{}: Bad syntax", num+1));
        }
        if parts[0].eq(username) {
            let start: uid_t = start.unwrap();
            let end = start + count.unwrap() - 1;
            res.push(Range::new(start, end));
        }
    }
    return Ok(res);
}

fn read_gid_map(username: &str) -> Result<Vec<Range>,String> {
    let file = File::open(&Path::new("/etc/subgid"));
    let mut res = Vec::new();
    let mut reader = BufferedReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try!(line.map_err(
            |e| format!("Error reading /etc/subgid: {}", e)));
        let parts: Vec<&str> = line.as_slice().split(':').collect();
        let start = FromStr::from_str(parts[1]);
        let count = FromStr::from_str(parts[2].trim_right());
        if parts.len() != 3 || start.is_none() || count.is_none() {
            return Err(format!("/etc/subgid:{}: Bad syntax", num+1));
        }
        if parts[0].eq(username) {
            let start: gid_t = start.unwrap();
            let end = start + count.unwrap() - 1;
            res.push(Range::new(start, end));
        }
    }
    return Ok(res);
}

pub fn match_ranges(req: &Vec<Range>, allowed: &Vec<Range>, own_id: uid_t)
    -> Vec<(uid_t, uid_t, uid_t)>
{
    let mut res = vec!((0, own_id, 1));
    let mut reqiter = req.iter();
    let mut reqval = *reqiter.next().unwrap();
    let mut allowiter = allowed.iter();
    let mut allowval = *allowiter.next().unwrap();
    loop {
        if reqval.start == 0 {
            reqval = reqval.shift(1);
        }
        if allowval.start == 0 {
            allowval = allowval.shift(1);
        }
        let clen = min(reqval.len(), allowval.len());
        if clen > 0 {
            res.push((reqval.start, allowval.start, clen));
        }
        reqval = reqval.shift(clen);
        allowval = allowval.shift(clen);
        if reqval.len() == 0 {
            reqval = match reqiter.next() {
                Some(val) => *val,
                None => break,
            };
        }
        if allowval.len() == 0 {
            allowval = match allowiter.next() {
                Some(val) => *val,
                None => unreachable!(),
            };
        }
    }
    return res;
}

pub fn get_max_uidmap() -> Result<Uidmap, String>
{
    let mut cmd = Command::new("id");
    cmd.arg("--user").arg("--name");
    if let Some(path) = getenv("HOST_PATH") {
        cmd.env("PATH", path);
    }
    cmd.stdin(Ignored).stderr(InheritFd(2));
    let username = try!(cmd.output()
        .map_err(|e| format!("Error running `id --user --name`: {}", e))
        .and_then(|out| if out.status == ExitStatus(0) { Ok(out.output) } else
            { Err(format!("Error running `id --user --name`")) })
        .and_then(|val| from_utf8(val.as_slice()).map(|x| x.trim().to_string())
                   .map_err(|e| format!("Can't decode username: {}", e))));
    let uid_map = read_uid_map(username.as_slice()).ok();
    let gid_map = read_gid_map(username.as_slice()).ok();

    let uid = unsafe { geteuid() };
    let gid = unsafe { getegid() };
    if let (Some(uid_map), Some(gid_map)) = (uid_map, gid_map) {
        let mut uids = vec!((0, uid, 1));
        for &rng in uid_map.iter() {
            let mut rng = rng;
            if uid >= rng.start && uid <= rng.end {
                // TODO(tailhook) implement better heuristic
                assert!(uid == rng.start);
                rng = rng.shift(1);
                if rng.len() == 0 { continue; }
            }
            uids.push((rng.start, rng.start, rng.len()));
        }

        let mut gids = vec!((0, gid, 1));
        for &rng in gid_map.iter() {
            let mut rng = rng;
            if gid >= rng.start && gid <= rng.end {
                // TODO(tailhook) implement better heuristic
                assert!(gid == rng.start);
                rng = rng.shift(1);
                if rng.len() == 0 { continue; }
            }
            gids.push((rng.start, rng.start, rng.len()));
        }

        return Ok(Ranges(uids, gids));
    } else {
        warn!(concat!("Your system doesn't have /etc/subuid and /etc/subgid.",
            " Presumably your system is too old. Some features may not work"));
        return Ok(Singleton(uid, gid));
    }
}

pub fn apply_uidmap(pid: pid_t, map: &Uidmap) -> Result<(), IoError> {
    match map {
        &Singleton(uid, gid) => {
            let uid_map = format!("0 {} 1", uid);
            debug!("Writing uid_map: {}", uid_map);
            match File::open_mode(&Path::new("/proc")
                              .join(pid.to_string())
                              .join("uid_map"), Open, Write)
                    .write_str(uid_map.as_slice()) {
                Ok(()) => {}
                Err(e) => return Err(IoError {
                    kind: e.kind,
                    desc: "Error writing uid mapping",
                    detail: e.detail,
                    }),
            }
            let gid_map = format!("0 {} 1", gid);
            debug!("Writing gid_map: {}", gid_map);
            match File::open_mode(&Path::new("/proc")
                              .join(pid.to_string())
                              .join("gid_map"), Open, Write)
                    .write_str(uid_map.as_slice()) {
                Ok(()) => {}
                Err(e) => return Err(IoError {
                    kind: e.kind,
                    desc: "Error writing gid mapping",
                    detail: e.detail
                    }),
            }
        }
        &Ranges(ref uids, ref gids) => {
            let myuid = unsafe { geteuid() };
            if myuid > 0 {
                let mut cmd = Command::new("newuidmap");
                cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
                cmd.arg(pid.to_string());
                for &(req, allowed, count) in uids.iter() {
                    cmd
                        .arg(req.to_string())
                        .arg(allowed.to_string())
                        .arg(count.to_string());
                }
                info!("Uid map command: {}", cmd);
                match cmd.status() {
                    Ok(ExitStatus(0)) => {},
                    Ok(ExitStatus(x)) => {
                        return Err(IoError {
                            kind: OtherIoError,
                            desc: "Error writing uid mapping",
                            detail: Some(format!(
                                "newuidmap exited with status {}", x)),
                            });
                    }
                    Ok(ExitSignal(x)) => {
                        return Err(IoError {
                            kind: OtherIoError,
                            desc: "Error writing uid mapping",
                            detail: Some(format!(
                                "newuidmap exited with signal {}", x)),
                            });
                    }
                    Err(e) => return Err(IoError {
                        kind: e.kind,
                        desc: "Error writing uid mapping",
                        detail: e.detail
                        }),
                }

                let mut cmd = Command::new("newgidmap");
                cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
                cmd.arg(pid.to_string());
                for &(req, allowed, count) in gids.iter() {
                    cmd
                        .arg(req.to_string())
                        .arg(allowed.to_string())
                        .arg(count.to_string());
                }
                info!("Gid map command: {}", cmd);
                match cmd.status() {
                    Ok(ExitStatus(0)) => {},
                    Ok(ExitStatus(x)) => {
                        return Err(IoError {
                            kind: OtherIoError,
                            desc: "Error writing gid mapping",
                            detail: Some(format!(
                                "newgidmap exited with status {}", x)),
                            });
                    }
                    Ok(ExitSignal(x)) => {
                        return Err(IoError {
                            kind: OtherIoError,
                            desc: "Error writing gid mapping",
                            detail: Some(format!(
                                "newgidmap exited with signal {}", x)),
                            });
                    }
                    Err(e) => return Err(IoError {
                        kind: e.kind,
                        desc: "Error writing gid mapping",
                        detail: e.detail
                        }),
                }
            } else {
                let mut membuf = MemWriter::new();
                for &(ins, outs, cnt) in uids.iter() {
                    try!(writeln!(&mut membuf, "{} {} {}", ins, outs, cnt));
                }
                let mut file = try!(File::create(&Path::new(
                    format!("/proc/{}/uid_map", pid))));
                let value = membuf.into_inner();
                debug!("Writing uid map ```{}```",
                    String::from_utf8_lossy(value.as_slice()));
                try!(file.write(value.as_slice()));

                let mut membuf = MemWriter::new();
                for &(ins, outs, cnt) in gids.iter() {
                    try!(writeln!(&mut membuf, "{} {} {}", ins, outs, cnt));
                }
                let mut file = try!(File::create(&Path::new(
                    format!("/proc/{}/gid_map", pid))));
                let value = membuf.into_inner();
                debug!("Writing gid map ```{}```",
                    String::from_utf8_lossy(value.as_slice()));
                try!(file.write(value.as_slice()));
            }
        }
    }
    return Ok(());
}

fn read_outside_ranges(path: &str) -> Result<Vec<Range>, String> {
    let mut file = BufferedReader::new(try!(File::open(&Path::new(path))
        .map_err(|e| format!("Error reading uid/gid map: {}", e))));
    let mut result = vec!();
    for line in file.lines() {
        let line = try!(line
            .map_err(|e| format!("Error reading uid/gid map: {}", e)));
        let mut words = line.as_slice().words();
        let outside = try!(words.next().and_then(FromStr::from_str)
            .ok_or(format!("uid/gid map format error")));
        try!(words.next()
            .ok_or(format!("uid/gid map format error")));
        let count = try!(words.next().and_then(FromStr::from_str)
            .ok_or(format!("uid/gid map format error")));
        result.push(Range { start: outside, end: outside+count-1 });
    }
    return Ok(result);
}

pub fn map_users(settings: &Settings, uids: &Vec<Range>, gids: &Vec<Range>)
    -> Result<Uidmap, String>
{
    let default_uids = vec!(Range { start: 0, end: 0 });
    let default_gids = vec!(Range { start: 0, end: 0 });
    let uids = if uids.len() > 0 { uids } else { &default_uids };
    let gids = if gids.len() > 0 { gids } else { &default_gids };
    if settings.uid_map.is_none() {
        let ranges = try!(read_outside_ranges("/proc/self/uid_map"));
        let uid_map = match_ranges(uids, &ranges, 0);
        let ranges = try!(read_outside_ranges("/proc/self/gid_map"));
        let gid_map = match_ranges(gids, &ranges, 0);
        return Ok(Ranges(uid_map, gid_map));
    } else {
        let &(ref uids, ref gids) = settings.uid_map.as_ref().unwrap();
        return Ok(Ranges(uids.clone(), gids.clone()));
    }
}
