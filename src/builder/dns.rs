use file_util::copy;

/// Reverts resolv.conf and hosts files after they might be replaces by
/// come container unpacking or copying
pub fn revert_name_files() -> Result<(), String> {
    copy("/etc/resolv.conf", "/vagga/root/etc/resolv.conf")
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e))?;

    copy("/etc/hosts", "/vagga/root/etc/hosts")
        .map_err(|e| format!("Error copying /etc/hosts: {}", e))?;

    Ok(())
}
