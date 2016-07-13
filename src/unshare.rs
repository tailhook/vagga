//! Dummy module used for mocking `unshare` crate on osx
use std::io;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd, IntoRawFd};
use std::fmt;
use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;
use std::ops::{Range, RangeTo, RangeFrom, RangeFull};

pub type pid_t = i32;
pub type uid_t = u32;
pub type gid_t = u32;
pub type SigNum = i32;

pub struct Command;
pub struct Stdio;
pub enum Fd {
    ReadNull,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UidMap {
    pub inside_uid: uid_t,
    pub outside_uid: uid_t,
    pub count: uid_t,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GidMap {
    pub inside_gid: gid_t,
    pub outside_gid: gid_t,
    pub count: gid_t,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Namespace {
    Mount,
    Uts,
    Ipc,
    User,
    Pid,
    Net,
}

pub struct Error;

pub struct Child {
    pub stdin: Option<File>,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}

pub enum AnyRange {
    RangeFrom(RawFd),
    Range(RawFd, RawFd),
}

pub struct ZombieIterator;
pub struct ChildEventsIterator;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(i8),
    Signaled(SigNum, /* dore dumped */bool)
}

pub enum ChildEvent {
    Death(pid_t, ExitStatus),
    Stop(pid_t, SigNum),
    Continue(pid_t),
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Command {
        unimplemented!();
    }
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command {
        unimplemented!();
    }
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut Command {
        unimplemented!();
    }

    pub fn init_env_map(&mut self) { unimplemented!(); }
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Command
        where K: AsRef<OsStr>, V: AsRef<OsStr>
    {
        unimplemented!();
    }

    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Command {
        unimplemented!();
    }

    pub fn env_clear(&mut self) -> &mut Command { unimplemented!(); }
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Command
    {
        unimplemented!();
    }

    pub fn stdin(&mut self, cfg: Stdio) -> &mut Command { unimplemented!(); }
    pub fn stdout(&mut self, cfg: Stdio) -> &mut Command { unimplemented!(); }
    pub fn stderr(&mut self, cfg: Stdio) -> &mut Command { unimplemented!(); }
    pub fn uid(&mut self, id: uid_t) -> &mut Command { unimplemented!(); }
    pub fn gid(&mut self, id: gid_t) -> &mut Command { unimplemented!(); }
    pub fn groups(&mut self, ids: Vec<gid_t>) -> &mut Command {
        unimplemented!();
    }
    pub fn allow_daemonize(&mut self) -> &mut Command { unimplemented!(); }
    pub fn set_parent_death_signal(&mut self, sig: SigNum) -> &mut Command {
        unimplemented!();
    }
    pub fn chroot_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Command
    {
        unimplemented!();
    }
    pub fn pivot_root<A: AsRef<Path>, B:AsRef<Path>>(&mut self,
        new_root: A, put_old: B, unmount: bool)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn unshare<I:IntoIterator<Item=Namespace>>(&mut self, iter: I)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn set_id_maps(&mut self, uid_map: Vec<UidMap>, gid_map: Vec<GidMap>)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn set_id_map_commands<A: AsRef<Path>, B: AsRef<Path>>(&mut self,
        newuidmap: A, newgidmap: B)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn keep_sigmask(&mut self) -> &mut Command {
        unimplemented!();
    }
    pub fn arg0<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command {
        unimplemented!();
    }
    pub fn make_group_leader(&mut self, make_group_leader: bool) -> &mut Command {
        unimplemented!();
    }
    pub fn status(&mut self) -> Result<ExitStatus, Error> { unimplemented!(); }
    pub fn spawn(&mut self) -> Result<Child, Error> { unimplemented!(); }
    pub fn file_descriptor(&mut self, target_fd: RawFd, cfg: Fd)
        -> &mut Command
    {
        unimplemented!();
    }
    pub unsafe fn file_descriptor_raw(&mut self, target_fd: RawFd, src: RawFd)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn close_fds<A: Into<AnyRange>>(&mut self, range: A)
        -> &mut Command
    {
        unimplemented!();
    }
    pub fn reset_fds(&mut self) -> &mut Command { unimplemented!(); }
}

impl ExitStatus {
    pub fn success(&self) -> bool { unimplemented!(); }
    pub fn code(&self) -> Option<i32> { unimplemented!(); }
    pub fn signal(&self) -> Option<i32> { unimplemented!(); }
}

impl Stdio {
    pub fn null() -> Stdio { unimplemented!(); }
    pub fn piped() -> Stdio { unimplemented!(); }
    pub fn inherit() -> Stdio { unimplemented!(); }
    pub fn from_raw_fd(_: i32) -> Stdio { unimplemented!(); }
    pub fn from_file<F: IntoRawFd>(file: F) -> Stdio { unimplemented!(); }
}

impl Namespace {
    pub fn to_clone_flag(&self) -> i32 { unimplemented!(); }
}


pub fn reap_zombies() -> ZombieIterator { unreachable!(); }
pub fn child_events() -> ChildEventsIterator { unreachable!(); }

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl fmt::Debug for ExitStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl fmt::Debug for Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl Child {
    pub fn id(&self) -> u32 { unimplemented!(); }
    pub fn pid(&self) -> pid_t { unimplemented!(); }
    pub fn wait(&mut self) -> Result<ExitStatus, io::Error> {
        unimplemented!();
    }
    pub fn signal(&self, signal: SigNum) -> Result<(), io::Error> {
        unimplemented!();
    }

    pub fn kill(&self) -> Result<(), io::Error> { unimplemented!(); }
    pub fn take_pipe_reader(&mut self, fd: RawFd) -> Option<File> {
        unimplemented!();
    }
    pub fn take_pipe_writer(&mut self, fd: RawFd) -> Option<File> {
        unimplemented!();
    }
}

impl Into<AnyRange> for Range<RawFd> {
    fn into(self) -> AnyRange {
        return AnyRange::Range(self.start, self.end);
    }
}

impl Into<AnyRange> for RangeTo<RawFd> {
    fn into(self) -> AnyRange {
        return AnyRange::Range(3, self.end);
    }
}

impl Into<AnyRange> for RangeFrom<RawFd> {
    fn into(self) -> AnyRange {
        return AnyRange::RangeFrom(self.start);
    }
}

impl Into<AnyRange> for RangeFull {
    fn into(self) -> AnyRange {
        return AnyRange::RangeFrom(3);
    }
}

impl Fd {
    pub fn piped_read() -> Fd { unimplemented!(); }
    pub fn piped_write() -> Fd { unimplemented!(); }
    pub fn inherit() -> Fd { unimplemented!(); }
    pub fn read_null() -> Fd { unimplemented!(); }
    pub fn write_null() -> Fd { Fd::ReadNull }
    pub fn dup_file<F: AsRawFd>(file: &F) -> io::Result<Fd> {
        unimplemented!();
    }
    pub fn from_file<F: IntoRawFd>(file: F) -> Fd { unimplemented!(); }
}

impl Iterator for ZombieIterator {
    type Item = (pid_t, ExitStatus);

    fn next(&mut self) -> Option<(pid_t, ExitStatus)> {
        unimplemented!();
    }
}

impl Iterator for ChildEventsIterator {
    type Item = ChildEvent;

    fn next(&mut self) -> Option<ChildEvent> {
        unimplemented!();
    }
}
