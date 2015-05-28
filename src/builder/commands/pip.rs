use std::old_io::BufferedReader;
use std::old_io::EndOfFile;
use std::old_io::fs::File;

use super::super::context::{BuildContext};
use super::super::packages;
use super::generic::{run_command_at_env, capture_command};


pub fn scan_features(ver: u8, pkgs: &Vec<String>) -> Vec<packages::Package> {
    let mut res = vec!();
    res.push(packages::BuildEssential);
    if ver == 2 {
        res.push(packages::Python2);
        res.push(packages::Python2Dev);
        res.push(packages::PipPy2);
    } else {
        res.push(packages::Python3);
        res.push(packages::Python3Dev);
        res.push(packages::PipPy3);
    }
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git+") {
            res.push(packages::Git);
        } else if name.as_slice().starts_with("hg+") {
            res.push(packages::Mercurial);
        }
    }
    return res;
}

fn pip_args(ctx: &mut BuildContext, ver: u8) -> Vec<String> {
    let mut args = vec!(
        (if ver == 2 { "python2" } else { "python3" }).to_string(),
        "-m".to_string(), "pip".to_string(),
        "install".to_string(),
        "--ignore-installed".to_string(),
        );
    if ctx.pip_settings.index_urls.len() > 0 {
        let mut indexes = ctx.pip_settings.index_urls.iter();
        if let Some(ref lnk) = indexes.next() {
            args.push(format!("--index-url={}", lnk));
            for lnk in indexes {
                args.push(format!("--extra-index-url={}", lnk));
            }
        }
    }
    ctx.pip_settings.trusted_hosts.iter().map(|h| {
        args.push("--trusted-host".to_string());
        args.push(h.to_string());
    }).last();
    if !ctx.pip_settings.dependencies {
        args.push("--no-deps".to_string());
    }
    for lnk in ctx.pip_settings.find_links.iter() {
        args.push(format!("--find-links={}", lnk));
    }
    return args;
}

pub fn pip_install(ctx: &mut BuildContext, ver: u8, pkgs: &Vec<String>)
    -> Result<(), String>
{
    try!(packages::ensure_packages(ctx, &scan_features(ver, pkgs)[0..]));
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.extend(pkgs.clone().into_iter());
    run_command_at_env(ctx, pip_cli.as_slice(), &Path::new("/work"), &[
        ("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
}

pub fn pip_requirements(ctx: &mut BuildContext, ver: u8, reqtxt: &Path)
    -> Result<(), String>
{
    let f = try!(File::open(&Path::new("/work").join(reqtxt))
        .map_err(|e| format!("Can't open requirements file: {}", e)));
    let mut f = BufferedReader::new(f);
    let mut names = vec!();
    loop {
        let line = match f.read_line() {
            Ok(line) => line,
            Err(ref e) if e.kind == EndOfFile => {
                break;
            }
            Err(e) => {
                return Err(format!("Error reading requirements: {}", e));
            }
        };
        let chunk = line.as_slice().trim();
        // Ignore empty lines and comments
        if chunk.len() == 0 || chunk.starts_with("#") {
            continue;
        }
        names.push(chunk.to_string());
    }

    try!(packages::ensure_packages(ctx, &scan_features(ver, &names)[0..]));
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.push("--requirement".to_string());
    pip_cli.push(reqtxt.display().to_string()); // TODO(tailhook) fix conversion
    run_command_at_env(ctx, pip_cli.as_slice(), &Path::new("/work"), &[
        ("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
}

pub fn configure(ctx: &mut BuildContext) -> Result<(), String> {
    try!(ctx.add_cache_dir(Path::new("/tmp/pip-cache"),
                           "pip-cache".to_string()));
    ctx.environ.insert("PIP_CACHE_DIR".to_string(),
                       "/tmp/pip-cache".to_string());
    Ok(())
}

pub fn freeze(ctx: &mut BuildContext) -> Result<(), String> {
    use std::fs::File;  // TODO(tailhook) migrate whole module
    use std::io::Write;  // TODO(tailhook) migrate whole module
    if ctx.featured_packages.contains(&packages::PipPy2) {
        try!(capture_command(ctx, &[
                "python2".to_string(),
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
            .and_then(|out| {
                File::create("/vagga/container/pip2-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            }));
    }
    if ctx.featured_packages.contains(&packages::PipPy3) {
        try!(capture_command(ctx, &[
                "python3".to_string(),
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
            .and_then(|out| {
                File::create("/vagga/container/pip3-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            }));
    }
    Ok(())
}
