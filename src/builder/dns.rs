use file_util::copy;
use std::fs::symlink_metadata;

/// Reverts resolv.conf and hosts files after they might be replaces by
/// come container unpacking or copying
pub fn revert_name_files() -> Result<(), String> {

    let md = symlink_metadata("/vagga/root/etc/resolv.conf");
    if md.is_err() || matches!(md, Ok(ref t) if t.is_file()) {
        copy("/etc/resolv.conf", "/vagga/root/etc/resolv.conf")
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e))?;
    } else {
        warn!("The `/etc/resolv.conf` is not a file, we avoid replacing it. \
               Under certain conditions this may mean that DNS does not work \
               in the container.")
    }

    let md = symlink_metadata("/vagga/root/etc/hosts");
    if md.is_err() || matches!(md, Ok(ref t) if t.is_file()) {
        copy("/etc/hosts", "/vagga/root/etc/hosts")
            .map_err(|e| format!("Error copying /etc/hosts: {}", e))?;
    } else {
        warn!("The `/etc/hosts` is not a file, we avoid replacing it. \
               Under certain conditions this may mean that DNS works badly \
               in the container.")
    }

    Ok(())
}
