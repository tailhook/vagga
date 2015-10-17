use std::io;
use std::path::PathBuf;

use unshare;

use config::builders::Builder;
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
        SubStep(step: Builder, err: Box<StepError>) {
            display("sub-step {:?} failed: {}", step, err)
        }
        /// Trying to run command failed because command is not found
        CommandNotFound(name: PathBuf, path: String) {
            display("command {:?} not found in one of {:?}", name, path)
        }
        /// Error starting external command
        CommandError(cmd: unshare::Command, err: unshare::Error) {
            display("error running {:?} {}", cmd, err)
        }
        /// Error running external command
        CommandFailed(cmd: unshare::Command, status: unshare::ExitStatus) {
            display("error running {:?} {}", cmd, status)
        }
        /// Can't open file, or similar
        Write(path: PathBuf, err: io::Error) {
            display("can't write file {:?}: {}", path, err)
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
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Step(step: Builder, err: StepError) {
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
