use std::path::Path;
use super::super::{vagga_cmd, check_status_output};


#[test]
fn test_symlink_fail() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/symlink_vagga"));
    vagga.arg("check");
    check_status_output(vagga, 255, "",
        "The `.vagga` dir can't be a symlink. Please run `unlink .vagga`\n");
}
