use super::Config;
use super::containers::Container;
use config::builders::Source as S;
use config::Step;

fn find_name(name: &str, cont: &Container, cfg: &Config) -> Result<(), String>
{
    for &Step(ref step) in &cont.setup {
        use config::builders::Builder::*;
        match step {
            &Container(ref subname) => {
                if subname == name {
                    return Err(format!("Container {:?} has cyclic dependency",
                                       name));
                } else {
                    let subcont = try!(cfg.containers.get(subname)
                        .ok_or(format!("Container {:?} referenced from {:?} \
                            is not found", subname, name)));
                    try!(find_name(name, subcont, cfg))
                }
            }
            &SubConfig(ref scfg) => {
                match scfg.source {
                    S::Container(ref subname) => {
                        if subname == name {
                            return Err(format!("Container {:?} has cyclic \
                                                dependency", name));
                        } else {
                            let subcont = try!(cfg.containers.get(subname)
                                .ok_or(format!("Container {:?} referenced \
                                    from {:?} is not found", subname, name)));
                            try!(find_name(name, subcont, cfg));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn validate_container(name: &str, cont: &Container, cfg: &Config)
    -> Result<(), String>
{
    try!(find_name(name, cont, cfg));
    Ok(())
}

pub fn validate_config(cfg: &Config) -> Result<(), String> {
    for (ref cname, ref cont) in &cfg.containers {
        try!(validate_container(cname, cont, cfg));
    }
    Ok(())
}
