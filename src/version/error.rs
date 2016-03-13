use std::io;
use std::path::PathBuf;

use regex;
use rustc_serialize::json;
use scan_dir;

quick_error! {
    /// Versioning error
    #[derive(Debug)]
    pub enum Error {
        /// Hash sum can't be calculated because some files need to be
        /// generated during build
        New {
            description("dependencies are not ready")
        }
        /// Some error occured. Unfortunately all legacy errors are strings
        String(s: String) {
            from()
            description("error versioning dependencies")
            display("version error: {}", s)
        }
        Regex(e: Box<regex::Error>) {
            description("can't compile regex")
            display("regex compilation error: {}", e)
        }
        ScanDir(errors: Vec<scan_dir::Error>) {
            from()
            description("can't read directory")
            display("error reading directory: {:?}", errors)
        }
        /// I/O error
        Io(err: io::Error, path: PathBuf) {
            cause(err)
            description("io error")
            display("Error reading {:?}: {}", path, err)
        }
        /// Container needed for build is not found
        ContainerNotFound(name: String) {
            description("container not found")
            display("container {:?} not found", name)
        }
        /// Some step of subcontainer failed
        SubStepError(step: String, err: Box<Error>) {
            from(tuple: (String, Error)) -> (tuple.0, Box::new(tuple.1))
        }
        /// Error reading package.json
        Json(err: json::BuilderError, path: PathBuf) {
            description("can't read json")
            display("error reading json {:?}: {:?}", path, err)
        }
    }
}
