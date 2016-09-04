use std::ascii::AsciiExt;
use std::collections::HashMap;

use docopt::{Docopt, Value, ArgvMap};

use launcher::user::ArgError;


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
    let opt = try!(Docopt::new(docopt)
        .map_err(|e| format!("Error parsing `options` in command {:?}: {}",
                             cmd, e))
        .map_err(ArgError::Error))
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
    for (key, value) in parsed.map.iter() {
        let key = key
            .trim_left_matches('-').trim_left_matches('<')
            .trim_right_matches('>');
        let env_var = format!("VAGGAOPT_{}",
            key.replace("-", "_").to_ascii_uppercase());
        match *value {
            Value::Switch(false) => env.insert(env_var, "".to_string()),
            Value::Switch(true) => env.insert(env_var, "true".to_string()),
            Value::Counted(0) => env.insert(env_var, "".to_string()),
            Value::Counted(v) => env.insert(env_var, format!("{}", v)),
            Value::Plain(None) => env.insert(env_var, "".to_string()),
            Value::Plain(Some(ref x))
            => env.insert(env_var, x.to_string()),
            Value::List(ref lst) => env.insert(env_var, lst.join(" ")),
        };
    }
    Ok((env, parsed))
}
