use std::os;
use std::io::stdio::{stdout, stderr};
use std::from_str::FromStr;

use argparse::{ArgumentParser, Store, List};
use collections::treemap::TreeMap;

use super::uidmap::write_uid_map;
use super::config::Range;
use super::monitor::Monitor;
use super::env::Environ;
use super::linux::{CPipe, run_newuser};
use super::options::env_options;


type IdRanges = Vec<Range>;


impl FromStr for IdRanges {
    fn from_str(src: &str) -> Option<IdRanges> {
        let mut res = Vec::new();
        for line in src.split(',') {
            match regex!(r"^(\d+)-(\d+)$").captures(line.as_slice()) {
                Some(caps) => {
                    res.push(Range::new(
                        from_str(caps.at(1)).unwrap(),
                        from_str(caps.at(2)).unwrap()));
                }
                None => {
                    match from_str(line) {
                        Some(x) => res.push(Range::new(x, x)),
                        None => return None,
                    }
                }
            }
        }
        return Some(res);
    }
}


pub fn run_userns(env: &mut Environ, args: Vec<String>) -> Result<int, String>
{
    let mut command: String = "".to_string();
    let mut cmdargs: Vec<String> = Vec::new();
    let mut uidranges: IdRanges = Vec::new();
    let mut gidranges: IdRanges = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut command)
            .add_argument("command", box Store::<String>,
                "A command to run inside container")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arguments", box List::<String>,
                "Arguments for the command");
        ap.refer(&mut uidranges)
            .add_option(["--uid-ranges"], box Store::<IdRanges>,
                "Uid ranges that must be mapped. E.g. 0-1000,65534");
        ap.refer(&mut gidranges)
            .add_option(["--gid-ranges"], box Store::<IdRanges>,
                "Gid ranges that must be mapped. E.g. 0-100,500-1000");
        env_options(env, &mut ap);
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }

    let mut runenv = TreeMap::new();
    for &(ref k, ref v) in os::env().iter() {
        runenv.insert(k.clone(), v.clone());
    }
    env.populate_environ(&mut runenv);

    let pipe = try!(CPipe::new());
    let mut monitor = Monitor::new(true);

    let pid = try!(run_newuser(&pipe, &command, cmdargs.as_slice(), &runenv));

    try!(write_uid_map(pid, &uidranges, &gidranges));

    try!(pipe.wakeup());

    monitor.add("child".to_string(), pid);
    monitor.wait_all();
    return Ok(monitor.get_status());
}
