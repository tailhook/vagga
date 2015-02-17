use std::io::BufferedReader;
use std::io::EndOfFile;
use std::io::fs::File;

use super::super::context::{BuildContext};
use super::generic::run_command;
use super::super::context::Distribution as Distr;
use super::super::dev::RevControl;
use super::debian;
use super::alpine;
use self::PipFeatures::*;


pub enum PipFeatures {
    Dev,
    Pip,
    Rev(RevControl),
}


pub fn ensure_pip(ctx: &mut BuildContext, ver: u8,
    features: &[PipFeatures])
    -> Result<Path, String>
{
    match ctx.distribution {
        Distr::Unknown => {
            return Err(format!("Unsupported distribution"));
        }
        Distr::Ubuntu(_) => {
            return debian::ensure_pip(ctx, ver, features);
        }
        Distr::Alpine(_) => {
            return alpine::ensure_pip(ctx, ver, features);
        }
    }
}

pub fn scan_features(pkgs: &Vec<String>) -> Vec<PipFeatures> {
    let mut res = vec!();
    res.push(Dev);
    res.push(Pip);
    for name in pkgs.iter() {
        if name.as_slice().starts_with("git+") {
            res.push(Rev(RevControl::Git));
        } else if name.as_slice().starts_with("hg+") {
            res.push(Rev(RevControl::Hg));
        }
    }
    return res;
}

fn pip_args(ctx: &mut BuildContext, pip_cmd: Path) -> Vec<String> {
    let mut args = vec!(
        pip_cmd.display().to_string(),  // TODO(tailhook) fix conversion
        "install".to_string(),
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
    let pip = try!(ensure_pip(ctx, ver, scan_features(pkgs).as_slice()));
    let mut pip_cli = pip_args(ctx, pip);
    pip_cli.extend(pkgs.clone().into_iter());
    run_command(ctx, pip_cli.as_slice())
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

    let pip = try!(ensure_pip(ctx, ver, scan_features(&names).as_slice()));
    let mut pip_cli = pip_args(ctx, pip);
    pip_cli.push("--requirement".to_string());
    pip_cli.push(reqtxt.display().to_string()); // TODO(tailhook) fix conversion
    run_command(ctx, pip_cli.as_slice())
}
