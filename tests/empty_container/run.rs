use std::path::Path;
use super::super::{vagga_cmd, check_status_output};


#[test]
fn test_hashsum() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/empty_container"));
    vagga.arg("_build");
    vagga.arg("empty");
    check_status_output(vagga, 0, "", concat!(
        ""
        ));
}
