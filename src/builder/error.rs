use std::io;
use std::rc::Rc;
use std::path::{Path, PathBuf};

use unshare;
use scan_dir;
use libmount;
use path_filter;

use build_step::BuildStep;
use builder::packages::Package;
use process_util::{cmd_debug};


quick_error! {
    #[derive(Debug)]
    pub enum StepError {
        DistroOverlap(new: &'static str, is: &'static str) {
            display("step initializes distribution {:?} but \
                the {:?} is already initialized", new, is)
        }
        WrongDistro(should: &'static str, is: &'static str) {
            display("step must be used on {:?} but \
                used on {:?} distribution instead", should, is)
        }
        NoDistro {
            display("this step requires some linux distribution to be active")
        }
        // Should substep include container name? or is it obvours?
        SubStep(step: Rc<BuildStep>, err: Box<StepError>) {
            display("sub-step {:?} failed: {}", step, err)
        }
        /// Trying to run command failed because command is not found
        CommandNotFound(name: PathBuf, path: String) {
            display("command {:?} not found in one of {:?}", name, path)
        }
        /// Error starting external command
        CommandError(cmd: Box<unshare::Command>, err: unshare::Error) {
            display("error running {} {}", cmd_debug(&cmd), err)
        }
        /// Error running external command
        CommandFailed(cmd: Box<unshare::Command>, status: unshare::ExitStatus) {
            display("error running {} {}", cmd_debug(&cmd), status)
        }
        /// Can't copy file
        Copy(src: PathBuf, dest: PathBuf, err: io::Error) {
            display("can't copy file {:?} -> {:?}: {}", src, dest, err)
            context(pair: (&'a Path, &'a Path), err: io::Error)
                -> (pair.0.to_path_buf(), pair.1.to_path_buf(), err)
            context(pair: (&'a PathBuf, &'a PathBuf), err: io::Error)
                -> (pair.0.clone(), pair.1.clone(), err)
        }
        /// Can't read file
        Read(path: PathBuf, err: io::Error) {
            display("can't read {:?}: {}", path, err)
        }
        /// Can't write file
        Write(path: PathBuf, err: io::Error) {
            display("can't write {:?}: {}", path, err)
        }
        /// Can't acquire lock
        Lock(msg: &'static str, err: io::Error) {
            display("{}: {}", msg, err)
        }
        /// Can't read directory for copying
        ScanDir(errors: Vec<scan_dir::Error>) {
            from()
            description("can't read directory")
            display("error reading directory: {:?}", errors)
        }
        /// Can't read directory for copying
        PathFilter(errors: Vec<path_filter::FilterError>) {
            from()
            description("can't read directory")
            display("error reading directory: {:?}", errors)
        }
        /// Distribution does not support the features
        UnsupportedFeatures(features: Vec<Package>) {
            display("current linux distribution does not support features: \
                {:?}", features)
        }
        /// Errors which have no data and may be presented to user just by
        /// text string
        Message(message: &'static str) {
            from()
            display("{}", message)
        }
        /// Compatibility error wrapper, should be removed in future
        Compat(message: String) {
            from()
            display("{}", message)
        }
        MountError(err: libmount::Error) {
            from()
            display("{}", err)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Step(step: Rc<BuildStep>, err: StepError) {
            display("step {:?} failed: {}", step, err)
        }
        /// Compatibility error wrapper, should be removed in future
        Compat(message: String) {
            from()
            display("{}", message)
        }
    }
}

impl From<StepError> for String {
    fn from(err: StepError) -> String {
        err.to_string()
    }
}
