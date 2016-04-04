use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::fmt::{Arguments};

use time::{self, Timespec};

const NANO_FACTOR: f64 = 0.000000001;


fn time_as_f64(tm: Timespec) -> f64 {
    (tm.sec as f64) + (NANO_FACTOR * tm.nsec as f64)
}

pub struct TimeLog {
    file: File,
    start: Timespec,
    prev: Timespec,
}


impl TimeLog {
    pub fn start(path: &Path) -> Result<TimeLog, Error> {
        let tm = time::get_time();
        let mut res = TimeLog {
            file: try!(File::create(path)),
            start: tm,
            prev: tm,
        };
        try!(res.mark(format_args!("Start {}", time_as_f64(tm))));
        Ok(res)
    }
    pub fn mark(&mut self, args: Arguments) -> Result<(), Error> {
        let tm = time::get_time();
        let tm_time = time_as_f64(tm);
        let tm_start = tm_time - time_as_f64(self.start);
        let tm_prev = tm_time - time_as_f64(self.prev);
        write!(&mut self.file, "{:7.3} {:7.3}   ", tm_start, tm_prev)
        .and_then(|()| self.file.write_fmt(args))
        .and_then(|()| writeln!(&mut self.file, ""))
        .map(|()| self.prev = tm)
    }
}
