use std::path::Path;
use super::super::{vagga_cmd, check_status_output, check_status_output_re};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/ubuntu"));
    vagga.arg("_version_hash");
    vagga.arg("ubuntu");
    check_status_output(vagga, 0,
        "b630929ffc800c4b2f578873d423f723840a650695af7f0bc084cdb56eda40b3\n",
        "");
}

#[test]
fn test_hashsum_echo() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/ubuntu"));
    vagga.arg("_version_hash");
    vagga.arg("ubuntu-echo");
    check_status_output(vagga, 0,
        "30ff6e7bfb547502e9c9d1061b778bde24393b58da316805a943610beee129fe\n",
        "");
}

#[test]
#[cfg(disabled_test)]
fn test_echo() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/ubuntu"));
    vagga.arg("_build");
    vagga.arg("ubuntu-echo");
    check_status_output_re(vagga, 0,
        &regex!("-{5} HELLO -{5}"),
        &regex!(""));
}
