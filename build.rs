use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("gcc").args(&["container.c", "-c", "-fPIC",
                               "-std=c99", "-D_GNU_SOURCE", "-o"])
                       .arg(&format!("{}/container.o", out_dir))
                       .status().unwrap();
    Command::new("ar").args(&["crus", "libcontainer.a", "container.o"])
                      .current_dir(&Path::new(&out_dir))
                      .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=container");
}
