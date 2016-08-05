use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::fmt::{Arguments};
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

const NANO_FACTOR: f64 = 0.000000001;


fn duration_as_f64(d: Duration) -> f64 {
    (d.as_secs() as f64) + (NANO_FACTOR * d.subsec_nanos() as f64)
}

pub struct TimeLog {
    file: File,
    start: Instant,
    prev: Instant,
}

impl TimeLog {
    pub fn start(path: &Path) -> Result<TimeLog, Error> {
        let now = SystemTime::now();
        let now_instant = Instant::now();
        let mut res = TimeLog {
            file: try!(File::create(path)),
            start: now_instant,
            prev: now_instant,
        };
        let duration = now.duration_since(UNIX_EPOCH)
                          .expect("FATAL: now was created after the epoch");
        try!(res.mark(format_args!("Start {:?}", duration_as_f64(duration))));
        Ok(res)
    }
    pub fn mark(&mut self, args: Arguments) -> Result<(), Error> {
        let now = Instant::now();
        let d_start = duration_as_f64(now.duration_since(self.start));
        let d_prev = duration_as_f64(now.duration_since(self.prev));
        write!(&mut self.file, "{:7.3} {:7.3}   ", d_start, d_prev)
        .and_then(|()| self.file.write_fmt(args))
        .and_then(|()| writeln!(&mut self.file, ""))
        .map(|()| self.prev = now)
    }
}
