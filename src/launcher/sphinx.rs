use std::io::{self, Write};
use config::Config;

pub fn title(out: &mut Write, ch: char, val: &str) -> Result<(), io::Error>{
    out.write(b"\n")?;
    writeln!(out, "{}", val)?;
    for _ in 0..val.len() {
        out.write(&[ch as u8])?;
    }
    out.write(b"\n\n")?;
    Ok(())
}

pub fn write_commands(out: &mut Write, config: &Config,
    hidden: bool, main_title: &str)
    -> Result<(), io::Error>
{
    title(out, '=', main_title)?;

    for (name, cmd) in &config.commands {
        if !hidden && name.starts_with("_") {
            continue
        }
        title(out, '-', name)?;
        writeln!(out, "{}\n",
            cmd.description().map(|x| &x[..]).unwrap_or("no description"))?;
    }
    Ok(())
}
