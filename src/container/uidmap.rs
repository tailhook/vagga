use std::cmp::min;
use std::cmp::Ordering;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::io::Error as IoError;
use std::io::ErrorKind::{Interrupted, Other};
use std::os::unix::process::ExitStatusExt;
use std::str::FromStr;
use std::str::from_utf8;
use std::path::Path;
use std::process::{Command, Stdio};

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
    let file = try_msg!(File::open(&Path::new("/etc/subuid")),
        "Can't open /etc/subuid: {err}");
    let mut res = Vec::new();
    let mut reader = BufReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try_msg!(line, "Error reading /etc/subuid: {err}");
        let parts: Vec<&str> = line[..].split(':').collect();
        let start = FromStr::from_str(parts[1]);
        let count: Result<uid_t, _> = FromStr::from_str(parts[2].trim_right());
        if parts.len() != 3 || start.is_err() || count.is_err() {
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
    let file = try_msg!(File::open(&Path::new("/etc/subgid")),
        "Can't open /etc/subgid: {err}");
    let mut res = Vec::new();
    let mut reader = BufReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try_msg!(line, "Error reading /etc/subgid: {err}");
        let parts: Vec<&str> = line[..].split(':').collect();
        let start = FromStr::from_str(parts[1]);
        let count: Result<uid_t, _> = FromStr::from_str(parts[2].trim_right());
        if parts.len() != 3 || start.is_err() || count.is_err() {
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
    -> Result<Vec<(uid_t, uid_t, uid_t)>, ()>
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
                None => return Err(()),
            };
        }
    }
    return Ok(res);
}

pub fn get_max_uidmap() -> Result<Uidmap, String>
{
    let mut cmd = Command::new("id");
    cmd.arg("--user").arg("--name");
    if let Ok(path) = env::var("HOST_PATH") {
        cmd.env("PATH", path);
    }
    cmd.stdin(Stdio::null()).stderr(Stdio::inherit());
    let username = try!(cmd.output()
        .map_err(|e| format!("Error running `id --user --name`: {}", e))
        .and_then(|out| if out.status.success() { Ok(out.stdout) } else
            { Err(format!("Error running `id --user --name`")) })
        .and_then(|val| from_utf8(&val).map(|x| x.trim().to_string())
                   .map_err(|e| format!("Can't decode username: {}", e))));
    let uid_map = read_uid_map(&username).ok();
    let gid_map = read_gid_map(&username).ok();

    let uid = unsafe { geteuid() };
    let gid = unsafe { getegid() };
    if let (Some(uid_map), Some(gid_map)) = (uid_map, gid_map) {
        if uid_map.len() == 0 && gid_map.len() == 0 && uid == 0 {
            let uid_rng = try!(read_uid_ranges("/proc/self/uid_map", true));
            let gid_rng = try!(read_uid_ranges("/proc/self/gid_map", true));
            return Ok(Ranges(
                uid_rng.into_iter()
                    .map(|r| (r.start, r.start, r.end-r.start+1)).collect(),
                gid_rng.into_iter()
                    .map(|r| (r.start, r.start, r.end-r.start+1)).collect()));
        }
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
            let uid_map_file = Path::new("/proc")
                              .join(pid.to_string())
                              .join("uid_map");
            try!(OpenOptions::new().write(true).open(&uid_map_file)
                .and_then(|mut f| f.write_all(uid_map.as_bytes())));
            let gid_map = format!("0 {} 1", gid);
            debug!("Writing gid_map: {}", gid_map);
            let gid_map_file = Path::new("/proc")
                              .join(pid.to_string())
                              .join("gid_map");
            try!(OpenOptions::new().write(true).open(&gid_map_file)
                .and_then(|mut f| f.write_all(uid_map.as_bytes())));
        }
        &Ranges(ref uids, ref gids) => {
            let myuid = unsafe { geteuid() };
            if myuid > 0 {
                let mut cmd = Command::new("newuidmap");
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
                cmd.arg(pid.to_string());
                for &(req, allowed, count) in uids.iter() {
                    cmd
                        .arg(req.to_string())
                        .arg(allowed.to_string())
                        .arg(count.to_string());
                }
                info!("Uid map command: {:?}", cmd);
                match cmd.status() {
                    Ok(s) if s.success() => {},
                    Ok(s) if s.signal().is_some() => {
                        return Err(IoError::new(Interrupted,
                            format!("Error writing uid mapping: \
                                newuidmap was killed on signal {}",
                                s.signal().unwrap())));
                    }
                    Ok(s) => {
                        return Err(IoError::new(Other,
                            format!("Error writing uid mapping: \
                                newuidmap was exit with status {}", s)));
                    }
                    Err(e) => return Err(e),
                }

                let mut cmd = Command::new("newgidmap");
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
                cmd.arg(pid.to_string());
                for &(req, allowed, count) in gids.iter() {
                    cmd
                        .arg(req.to_string())
                        .arg(allowed.to_string())
                        .arg(count.to_string());
                }
                info!("Gid map command: {:?}", cmd);
                match cmd.status() {
                    Ok(s) if s.success() => {},
                    Ok(s) if s.signal().is_some() => {
                        return Err(IoError::new(Interrupted,
                            format!("Error writing gid mapping: \
                                newuidmap was killed on signal {}",
                                s.signal().unwrap())));
                    }
                    Ok(s) => {
                        return Err(IoError::new(Other,
                            format!("Error writing gid mapping: \
                                newuidmap was exit with status {}", s)));
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let mut membuf = Vec::with_capacity(100);
                for &(ins, outs, cnt) in uids.iter() {
                    try!(writeln!(&mut membuf, "{} {} {}", ins, outs, cnt));
                }
                let mut file = try!(File::create(
                    &format!("/proc/{}/uid_map", pid)));
                debug!("Writing uid map ```{}```",
                    String::from_utf8_lossy(&membuf[..]));
                try!(file.write(&membuf));

                let mut membuf = Vec::with_capacity(100);
                for &(ins, outs, cnt) in gids.iter() {
                    try!(writeln!(&mut membuf, "{} {} {}", ins, outs, cnt));
                }
                let mut file = try!(File::create(
                    &format!("/proc/{}/gid_map", pid)));
                debug!("Writing gid map ```{}```",
                    String::from_utf8_lossy(&membuf[..]));
                try!(file.write(&membuf));
            }
        }
    }
    return Ok(());
}

fn read_uid_ranges(path: &str, read_inside: bool) -> Result<Vec<Range>, String>
{
    let mut file = BufReader::new(try!(File::open(&Path::new(path))
        .map_err(|e| format!("Error reading uid/gid map: {}", e))));
    let mut result = vec!();
    for line in file.lines() {
        let line = try!(line
            .map_err(|e| format!("Error reading uid/gid map")));
        let mut words = line[..].split_whitespace();
        let inside: u32 = try!(words.next().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("uid/gid map format error")));
        let outside: u32 = try!(words.next().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("uid/gid map format error")));
        let count: u32 = try!(words.next().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("uid/gid map format error")));
        if read_inside {
            result.push(Range { start: inside, end: inside+count-1 });
        } else {
            result.push(Range { start: outside, end: outside+count-1 });
        }
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
        let ranges = try!(read_uid_ranges("/proc/self/uid_map", true));
        let uid_map = try!(match_ranges(uids, &ranges, 0)
            .map_err(|()| {
                return format!("Number of allowed subuids is too small. \
                    Required {:?}, allowed {:?}. You either need to increase \
                    allowed numbers in /etc/subuid (preferred) or decrease \
                    needed ranges in vagga.yaml by adding `uids` key \
                    to container config", uids, ranges);
            }));
        let ranges = try!(read_uid_ranges("/proc/self/gid_map", true));
        let gid_map = try!(match_ranges(gids, &ranges, 0)
            .map_err(|()| {
                return format!("Number of allowed subgids is too small. \
                    Required {:?}, allowed {:?}. You either need to increase \
                    allowed numbers in /etc/subgid (preferred) or decrease \
                    needed ranges in vagga.yaml by adding `gids` key \
                    to container config", gids, ranges);
            }));
        return Ok(Ranges(uid_map, gid_map));
    } else {
        let &(ref uids, ref gids) = settings.uid_map.as_ref().unwrap();
        return Ok(Ranges(uids.clone(), gids.clone()));
    }
}
