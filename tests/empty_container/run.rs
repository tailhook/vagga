use std::path::Path;
use super::super::{vagga_cmd, check_status_output};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/empty_container"));
    vagga.arg("_version_hash");
    vagga.arg("empty");
    check_status_output(vagga, 0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "");
}
