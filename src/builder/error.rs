use std::io;
use std::rc::Rc;
use std::path::PathBuf;

use unshare;
use regex;
use scan_dir;
use libmount;

use build_step::BuildStep;
use builder::packages::Package;


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
            display("error running {:?} {}", cmd, err)
        }
        /// Error running external command
        CommandFailed(cmd: Box<unshare::Command>, status: unshare::ExitStatus) {
            display("error running {:?} {}", cmd, status)
        }
        /// Can't open file, or similar
        Write(path: PathBuf, err: io::Error) {
            display("can't write file {:?}: {}", path, err)
        }
        /// Can't read directory for copying
        ScanDir(errors: Vec<scan_dir::Error>) {
            from()
            description("can't read directory")
            display("error reading directory: {:?}", errors)
        }
        /// Can't compile regex
        Regex(e: Box<regex::Error>) {
            description("can't compile regex")
            display("error compiling regex: {}", e)
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
