use std::path::Path;
use super::super::{vagga_cmd, check_status_output, check_status_output_re};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/empty_container"));
    vagga.arg("_version_hash");
    vagga.arg("empty");
    check_status_output(vagga, 0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        "");
}

#[test]
fn test_build() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/empty_container"));
    vagga.arg("_build");
    vagga.arg("empty");
    check_status_output(vagga, 0, "", "");
}

#[test]
fn test_build_echo_error() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/empty_container"));
    vagga.arg("_build");
    vagga.arg("empty-echo");
    check_status_output(vagga, 1, "",
        "ERROR:main: Error build command !Sh echo hello: \
         Command [/bin/sh, -c, echo hello] exited with status 127\n");
}
