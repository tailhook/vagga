use super::super::context::BuildContext;
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;


pub fn fetch_ubuntu_core(ctx: &mut BuildContext, release: &String)
    -> Result<(), String>
{
    let kind = "core";
    let arch = "amd64";
    let url = format!(concat!(
        "http://cdimage.ubuntu.com/ubuntu-{kind}/{release}/",
        "daily/current/{release}-{kind}-{arch}.tar.gz",
        ), kind=kind, arch=arch, release=release);
    let filename = try!(download_file(ctx, &url));
    try!(unpack_file(ctx, &filename, &Path::new("/vagga/root")));
    try!(init_debian_build(ctx));
    return Ok(());
}

fn init_debian_build(ctx: &mut BuildContext) -> Result<(), String> {
    try!(ctx.add_cache_dir(Path::new("/var/cache/apt"),
                           "apt-cache".to_string()));
    // TODO(tailhook) remove apt and dpkg
    ctx.add_remove_dir(Path::new("/var/lib/apt"));
    ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    return Ok(());
}
