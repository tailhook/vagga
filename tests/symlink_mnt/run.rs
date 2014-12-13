use std::path::Path;
use super::super::{vagga_cmd, check_status_output_re};


#[test]
fn test_symlink_fail() {
    let mut vagga = vagga_cmd();
    vagga.cwd(&Path::new("tests/symlink_mnt"));
    vagga.arg("_build");
    vagga.arg("empty");
    check_status_output_re(vagga, 122, &regex!(r"^$"), &regex!(concat!(
        "^The `[^`]+.vagga/.mnt` dir can't be a symlink. ",
        "Please run `unlink [^`]+.vagga/.mnt`\n$")));
}
