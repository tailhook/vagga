use std::path::Path;
use super::super::{vagga_cmd, check_status_output};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/ubuntu"));
    vagga.arg("_build");
    vagga.arg("ubuntu");
    check_status_output(vagga, 0, "", concat!(
        ""
        ));
}
