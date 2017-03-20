use std::collections::BTreeMap;
use std::env;
use std::io::{stdout, stderr};
use std::os::unix::fs::symlink;
use std::path::Path;

use argparse::{ArgumentParser};
use unshare::{Command};
use rustc_serialize::json;

use config::command::CapsuleInfo;

use super::setup;
use super::Wrapper;
use super::util::{find_cmd};
use file_util::Dir;
use process_util::{run_and_wait, convert_status, copy_env_vars};


const BUSYBOX_COMMANDS: &'static [&'static str] = &[
    "[", "[[", "acpid", "add-shell", "addgroup", "adduser", "adjtimex", "arp",
    "arping", "ash", "awk", "base64", "basename", "bbconfig", "beep",
    "blkdiscard", "blkid", "blockdev", "brctl", "bunzip2", "bzcat", "bzip2",
    "cal", "cat", "catv", "chgrp", "chmod", "chown", "chpasswd", "chroot",
    "chvt", "cksum", "clear", "cmp", "comm", "conspy", "cp", "cpio", "crond",
    "crontab", "cryptpw", "cut", "date", "dc", "dd", "deallocvt", "delgroup",
    "deluser", "depmod", "df", "diff", "dirname", "dmesg", "dnsd",
    "dnsdomainname", "dos2unix", "du", "dumpkmap", "dumpleases", "echo", "ed",
    "egrep", "eject", "env", "ether-wake", "expand", "expr", "fakeidentd",
    "false", "fatattr", "fbset", "fbsplash", "fdflush", "fdformat", "fdisk",
    "fgrep", "find", "findfs", "flock", "fold", "free", "fsck", "fstrim",
    "fsync", "ftpd", "ftpget", "ftpput", "fuser", "getopt", "getty", "grep",
    "groups", "gunzip", "gzip", "halt", "hd", "hdparm", "head", "hexdump",
    "hostid", "hostname", "httpd", "hwclock", "id", "ifconfig", "ifdown",
    "ifenslave", "ifup", "inetd", "init", "inotifyd", "insmod", "install",
    "ionice", "iostat", "ip", "ipaddr", "ipcalc", "ipcrm", "ipcs", "iplink",
    "iproute", "iprule", "iptunnel", "kbd_mode", "kill", "killall",
    "killall5", "klogd", "less", "ln", "loadfont", "loadkmap", "logger",
    "login", "logread", "losetup", "ls", "lsmod", "lsof", "lspci", "lsusb",
    "lzcat", "lzma", "lzop", "lzopcat", "makemime", "md5sum", "mdev", "mesg",
    "microcom", "mkdir", "mkdosfs", "mkfifo", "mkfs.vfat", "mknod",
    "mkpasswd", "mkswap", "mktemp", "modinfo", "modprobe", "more", "mount",
    "mountpoint", "mpstat", "mv", "nameif", "nanddump", "nandwrite",
    "nbd-client", "nc", "netstat", "nice", "nmeter", "nohup", "nologin",
    "nsenter", "nslookup", "ntpd", "od", "openvt", "passwd", "patch", "pgrep",
    "pidof", "ping", "ping6", "pipe_progress", "pkill", "pmap", "poweroff",
    "powertop", "printenv", "printf", "ps", "pscan", "pstree", "pwd", "pwdx",
    "raidautorun", "rdate", "rdev", "readahead", "readlink", "readprofile",
    "realpath", "reboot", "reformime", "remove-shell", "renice", "reset",
    "resize", "rev", "rfkill", "rm", "rmdir", "rmmod", "route", "run-parts",
    "sed", "sendmail", "seq", "setconsole", "setfont", "setkeycodes",
    "setlogcons", "setserial", "setsid", "sh", "sha1sum", "sha256sum",
    "sha3sum", "sha512sum", "showkey", "shuf", "slattach", "sleep", "smemcap",
    "sort", "split", "stat", "strings", "stty", "su", "sum", "swapoff",
    "swapon", "switch_root", "sync", "sysctl", "syslogd", "tac", "tail",
    "tar", "tee", "telnet", "test", "tftp", "time", "timeout", "top", "touch",
    "tr", "traceroute", "traceroute6", "true", "truncate", "tty", "ttysize",
    "tunctl", "udhcpc", "udhcpc6", "udhcpd", "umount", "uname", "unexpand",
    "uniq", "unix2dos", "unlink", "unlzma", "unlzop", "unshare", "unxz",
    "unzip", "uptime", "usleep", "uudecode", "uuencode", "vconfig", "vi",
    "vlock", "volname", "watch", "watchdog", "wc", "wget", "which", "whoami",
    "whois", "xargs", "xzcat", "yes", "zcat"
];

#[derive(RustcEncodable, Clone, Debug)]
pub struct Settings<'a> {
    pub version_check: bool,
    pub proxy_env_vars: bool,
    pub ubuntu_mirror: &'a Option<String>,
    pub alpine_mirror: &'a Option<String>,
    pub build_lock_wait: bool,
    pub environ: &'a BTreeMap<String, String>,
    pub index_all_images: bool,
    pub run_symlinks_as_commands: bool,
}


pub fn symlink_busybox_commands() -> Result<(), String> {
    Dir::new("/bin").create()
        .map_err(|e| format!("Can't create /bin: {}", e))?;
    for cmd in BUSYBOX_COMMANDS {
        symlink("/vagga/bin/busybox", format!("/bin/{}", cmd))
            .map_err(|e| format!("Error symlinking {:?}: {}", cmd, e))?;
    }
    Ok(())
}

pub fn commandline_cmd(_cmd_name: &str, command: &CapsuleInfo,
    wrapper: &Wrapper, mut cmdline: Vec<String>)
    -> Result<i32, String>
{
    if command.run.len() == 0 {
        return Err(format!(
            r#"Command has empty "run" parameter. Nothing to run."#));
    }
    // TODO(tailhook) detect other shells too
    let has_args = command.accepts_arguments
            .unwrap_or(&command.run[0][..] != "/bin/sh");
    let mut args = Vec::new();
    if !has_args {
        let mut ap = ArgumentParser::new();
        ap.set_description(command.description.as_ref()
            .map(|x| &x[..]).unwrap_or(""));
        ap.stop_on_first_argument(true);
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    } else {
        cmdline.remove(0);
        args.extend(cmdline.into_iter());
    }
    let mut cmdline = command.run.clone();
    cmdline.extend(args.into_iter());

    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    symlink_busybox_commands()?;

    let env = setup::get_capsule_environment(&wrapper.settings, command)?;
    let cpath = find_cmd(&cmdline.remove(0), &env)?;

    let mut cmd = Command::new(&cpath);
    cmd.args(&cmdline);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    let ref s = wrapper.settings;
    cmd.env("VAGGA_SETTINGS", json::encode(&Settings {
            version_check: s.version_check,
            proxy_env_vars: s.proxy_env_vars,
            ubuntu_mirror: &s.ubuntu_mirror,
            alpine_mirror: &s.alpine_mirror,
            build_lock_wait: s.build_lock_wait,
            environ: &s.environ,
            index_all_images: s.index_all_images,
            run_symlinks_as_commands: s.run_symlinks_as_commands,
        }).unwrap());
    if let Some(ref wd) = command.work_dir {
        cmd.current_dir(Path::new("/work").join(&wd));
    } else {
        cmd.current_dir(env::var("_VAGGA_WORKDIR")
                        .unwrap_or("/work".to_string()));
    }
    for (ref k, ref v) in env.iter() {
        cmd.env(k, v);
    }

    return run_and_wait(&mut cmd).map(convert_status);
}
