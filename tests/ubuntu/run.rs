use std::path::Path;
use super::super::{vagga_cmd, check_status_output};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/ubuntu"));
    vagga.arg("_version_hash");
    vagga.arg("ubuntu");
    check_status_output(vagga, 0,
        "b630929ffc800c4b2f578873d423f723840a650695af7f0bc084cdb56eda40b3",
        "");
}
