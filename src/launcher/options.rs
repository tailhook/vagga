use std::fmt;
use std::fmt::Write;
use std::iter::repeat;
use std::collections::HashMap;

use docopt::{Docopt, Value, ArgvMap};

use launcher::user::ArgError;

struct Escaped<T: AsRef<str>>(T);

impl<T: AsRef<str>> fmt::Display for Escaped<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let data = self.0.as_ref();
        if data.len() == 0 {
            write!(fmt, "''")
        } else if !data.contains(|c: char| !(c.is_alphanumeric() ||
            matches!(c, '@'|'%'|'+'|'='|':'|','|'.'|'/'|'-')))
        {
            write!(fmt, "{}", data)
        } else {
            fmt.write_str("'")?;
            for c in data.chars() {
                if c == '\'' {
                    fmt.write_str(r#"'"'"'"#)?;
                } else {
                    write!(fmt, "{}", c)?;
                }
            }
            fmt.write_str("'")
        }
    }
}


pub fn parse_docopts(description: &Option<String>,
    original_text: &str, extra_text: &str,
    cmd: &str, mut args: Vec<String>)
    -> Result<(HashMap<String, String>, ArgvMap), ArgError>
{
    args.insert(0, "vagga".to_string());
    args.insert(1, cmd.to_string());
    let docopt = format!("\
            {}\n\
            \n\
            {}\n\
            {}\n",
        description.as_ref().unwrap_or(&format!("vagga {}", cmd)),
        original_text,
        extra_text);
    let opt = Docopt::new(docopt)
        .map_err(|e| format!("Error parsing `options` in command {:?}: {}",
                             cmd, e))
        .map_err(ArgError::Error)?
        .argv(args);
    let parsed = match opt.parse() {
        Ok(parsed) => parsed,
        Err(ref e) if e.fatal() => {
            return Err(ArgError::Error(format!("{}", e)));
        }
        Err(ref e) => {
            println!("{}", e);
            return Err(ArgError::Exit(0));
        }
    };
    let mut env = HashMap::new();
    for (orig_key, value) in parsed.map.iter() {
        let key = orig_key
            .trim_left_matches('-').trim_left_matches('<')
            .trim_right_matches('>');
        let env_var = format!("VAGGAOPT_{}",
            key.replace("-", "_").to_ascii_uppercase());
        let env_var2 = format!("VAGGACLI_{}",
            key.replace("-", "_").to_ascii_uppercase());
        match *value {
            Value::Switch(false) => {
                env.insert(env_var, "".to_string());
                env.insert(env_var2, "".to_string());
            }
            Value::Switch(true) => {
                env.insert(env_var, "true".to_string());
                env.insert(env_var2, orig_key.to_string());
            }
            Value::Counted(0) => {
                env.insert(env_var, "".to_string());
                env.insert(env_var2, "".to_string());
            }
            Value::Counted(v) => {
                env.insert(env_var, format!("{}", v));
                if orig_key.starts_with("--") {  // long option
                    env.insert(env_var2,
                        repeat(&orig_key[..]).take(v as usize)
                            .collect::<Vec<_>>().join(" "));
                } else {  // short option
                    env.insert(env_var2,
                        format!("-{}",
                            repeat(&orig_key[1..2]).take(v as usize)
                            .collect::<String>()));
                }
            }
            Value::Plain(None) => {
                env.insert(env_var, "".to_string());
                env.insert(env_var2, "".to_string());
            }
            Value::Plain(Some(ref x)) => {
                env.insert(env_var, x.to_string());
                if orig_key.starts_with('-') {
                    env.insert(env_var2, format!("{} {}", orig_key,
                        Escaped(x)));
                } else {
                    env.insert(env_var2, format!("{}", Escaped(x)));
                }
            }
            Value::List(ref lst) => {
                if lst.len() > 0 {
                    env.insert(env_var, lst.join(" "));
                    let mut buf = String::new();
                    if orig_key.starts_with('-') {
                        write!(&mut buf, "{}", orig_key).unwrap();
                    }
                    for value in lst {
                        write!(&mut buf, " {}", Escaped(value)).unwrap();
                    }
                    env.insert(env_var2, buf);
                } else {
                    env.insert(env_var, "".to_string());
                    env.insert(env_var2, "".to_string());
                }
            }
        }
    }
    Ok((env, parsed))
}
