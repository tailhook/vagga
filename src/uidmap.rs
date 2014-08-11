use std::io::{File, Open, Write};
use std::io::BufferedReader;
use std::cmp::min;
use std::iter::AdditiveIterator;
use std::io::process::{ExitStatus, Command, Ignored, InheritFd};

use libc::funcs::posix88::unistd::{geteuid, getegid};
use libc::pid_t;

use super::config::Range;
use super::linux::get_user_name;


fn read_uid_map() -> Result<Vec<Range>,String> {
    let uid = unsafe { geteuid() };
    let user = try!(get_user_name(uid)
        .map_err(|e| format!("Error getting user name for {}: {}", uid, e)));
    let file = File::open(&Path::new("/etc/subuid"));
    let mut res = Vec::new();
    let mut reader = BufferedReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try!(line.map_err(
            |e| format!("Error reading /etc/subuid: {}", e)));
        let parts: Vec<&str> = line.as_slice().split(':').collect();
        let start = from_str(*parts.get(1));
        let count = from_str(*parts.get(2));
        if parts.len() != 3 || start.is_none() || count.is_none() {
            return Err(format!("/etc/newuid:{}: Bad syntax", num+1));
        }
        if parts.get(0).eq(&user.as_slice()) {
            let start: uint = start.unwrap();
            let end = start + count.unwrap() - 1;
            res.push(Range::new(start, end));
        }
    }
    return Ok(res);
}

fn read_gid_map() -> Result<Vec<Range>,String> {
    let uid = unsafe { geteuid() };
    let user = try!(get_user_name(uid)
        .map_err(|e| format!("Error getting user name for {}: {}", uid, e)));
    let file = File::open(&Path::new("/etc/subgid"));
    let mut res = Vec::new();
    let mut reader = BufferedReader::new(file);
    for (num, line) in reader.lines().enumerate() {
        let line = try!(line.map_err(
            |e| format!("Error reading /etc/subgid: {}", e)));
        let parts: Vec<&str> = line.as_slice().split(':').collect();
        let start = from_str(*parts.get(1));
        let count = from_str(*parts.get(2));
        if parts.len() != 3 || start.is_none() || count.is_none() {
            return Err(format!("/etc/subgid:{}: Bad syntax", num+1));
        }
        if parts.get(0).eq(&user.as_slice()) {
            let start: uint = start.unwrap();
            let end = start + count.unwrap() - 1;
            res.push(Range::new(start, end));
        }
    }
    return Ok(res);
}

pub fn match_ranges(req: &Vec<Range>, allowed: &Vec<Range>)
    -> Vec<(uint, uint, uint)>
{
    let mut res = Vec::new();
    let mut reqiter = req.iter();
    let mut reqval = *reqiter.next().unwrap();
    let mut allowiter = allowed.iter();
    let mut allowval = *reqiter.next().unwrap();
    loop {
        let clen = min(reqval.len(), allowval.len());
        res.push((reqval.start, allowval.start, clen));
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

pub fn write_uid_map(pid: pid_t, uid_req: &Vec<Range>, gid_req: &Vec<Range>)
    -> Result<(), String>
{
    let uid = unsafe { geteuid() };
    let gid = unsafe { getegid() };
    if uid_req.len() == 0 && gid_req.len() == 0 {
        //  No required mapping, just write our effective uid/gid to mapping
        let uid_map = format!("0 {} 1", uid);
        debug!("Writing uid_map: {}", uid_map);
        match File::open_mode(&Path::new("/proc")
                          .join(pid.to_str())
                          .join("uid_map"), Open, Write)
                .write_str(uid_map.as_slice()) {
            Ok(()) => {}
            Err(e) => return Err(format!(
                "Error writing uid mapping: {}", e)),
        }

        let gid_map = format!("0 {} 1", gid);
        debug!("Writing gid_map: {}", gid_map);
        match File::open_mode(&Path::new("/proc")
                          .join(pid.to_str())
                          .join("gid_map"), Open, Write)
                .write_str(gid_map.as_slice()) {
            Ok(()) => {}
            Err(e) => return Err(format!(
                "Error writing gid mapping: {}", e)),
        }
    } else {

        let allowed_uids = try!(read_uid_map());
        let allowed_gids = try!(read_gid_map());

        if uid_req.len() > 5 {
            warn!(concat!("Probably too many uid ranges required. ",
                          "Kernel might support no more than 5"));
        }
        if gid_req.len() > 5 {
            warn!(concat!("Probably too many gid ranges required. ",
                          "Kernel might support no more than 5"));
        }

        let usum = uid_req.iter().map(|r| r.len()).sum();
        let ausum = allowed_uids.iter().map(|r| r.len()).sum();

        let gsum = gid_req.iter().map(|r| r.len()).sum();
        let agsum = allowed_gids.iter().map(|r| r.len()).sum();

        if usum > ausum || gsum > agsum {
            warn!(concat!("Container requires more users or group than you ",
                          "allowed to use (see /etc/subuid, /etc/subgid)"));
        }

        let uid_map = match_ranges(uid_req, &allowed_uids);
        let gid_map = match_ranges(gid_req, &allowed_gids);

        debug!("Uid map: {}", uid_map);
        debug!("Gid map: {}", gid_map);

        let mut cmd = Command::new("newuidmap");
        cmd.stdin(Ignored).stdout(InheritFd(0)).stderr(InheritFd(2));
        cmd.arg(pid.to_str());
        for &(req, act, count) in uid_map.iter() {
            cmd.arg(act.to_str()).arg(req.to_str()).arg(count.to_str());
        }
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            x => return Err(format!("Error running newuidmap: {}", x)),
        }

        let mut cmd = Command::new("newgidmap");
        cmd.stdin(Ignored).stdout(InheritFd(0)).stderr(InheritFd(2));
        cmd.arg(pid.to_str());
        for &(req, act, count) in gid_map.iter() {
            cmd.arg(act.to_str()).arg(req.to_str()).arg(count.to_str());
        }
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            x => return Err(format!("Error running newgidmap: {}", x)),
        }
    }
    return Ok(());
}
