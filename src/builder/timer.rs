use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::fmt::{Arguments};

use time::get_time;


pub struct TimeLog {
    file: File,
    start: Timespec,
    prev: Timespec,
}


impl TimeLog {
    pub fn start(path: &Path) -> Result<TimeLog, Error> {
        let tm = get_time();
        let mut res =TimeLog {
            file: try!(File::create(path)),
            start: tm,
            prev: tm,
        };
        try!(res.mark(format_args!("Start {}", tm)));
        Ok(res)
    }
    pub fn mark(&mut self, args: Arguments) -> Result<(), Error> {
        let tm = get_time();
        write!(&mut self.file,
               "{:7.3} {:7.3}   ", tm - self.start, tm - self.prev)
        .and_then(|()| self.file.write_fmt(args))
        .and_then(|()| writeln!(&mut self.file, ""))
        .map(|()| self.prev = tm)
    }
}
