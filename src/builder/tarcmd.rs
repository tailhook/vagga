use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::context::BuildContext;


pub fn unpack_file(_ctx: &mut BuildContext, src: &Path, tgt: &Path,
    includes: &[Path])
    -> Result<(), String>
{
    info!("Unpacking {} -> {}", src.display(), tgt.display());
    let mut cmd = Command::new("/vagga/bin/busybox");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
        .arg("tar")
        .arg("-x")
        .arg("-f").arg(src)
        .arg("-C").arg(tgt);
    for i in includes.iter() {
        cmd.arg(i);
    }

    match src.extension_str() {
        Some("gz")|Some("tgz") => { cmd.arg("-z"); }
        Some("bz")|Some("tbz") => { cmd.arg("-j"); }
        Some("xz")|Some("txz") => { cmd.arg("-J"); }
        _ => {}
    };
    debug!("Running: {}", cmd);
    match cmd.output()
        .map_err(|e| format!("Can't run tar: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => Ok(()),
        Ok(val) => Err(format!("Tar exited with status: {}", val)),
        Err(x) => Err(format!("Error running tar: {}", x)),
    }
}
