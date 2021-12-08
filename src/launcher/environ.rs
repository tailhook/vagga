use std::env;
use std::collections::BTreeSet;

use regex::{Regex, escape};

use failure::{Error, err_msg, ResultExt};

use crate::config::read_settings::MergedSettings;


fn patterns_to_regex(patterns: &BTreeSet<String>) -> Result<Regex, Error> {
    let mut var_pattern = String::with_capacity(100);
    for item in patterns {
        if var_pattern.len() > 0 {
            var_pattern.push('|');
        }
        var_pattern.push('^');
        var_pattern.push_str(&escape(item).replace(r"\*", ".*"));
        var_pattern.push('$');
    }
    debug!("Propagation pattern: {:?}", var_pattern);
    Ok(Regex::new(&var_pattern)?)
}


pub fn set_initial_vaggaenv_vars(
    propagate_env: Vec<String>, set_env: Vec<String>,
    settings: &MergedSettings)
    -> Result<(), Error>
{
    for k in propagate_env.into_iter() {
        if k.chars().find(|&c| c == '=').is_some() {
            return Err(err_msg("Environment variable name \
                (for option `-e`/`--use-env`) \
                can't contain equals `=` character. \
                To set key-value pair use `-E`/`--environ` option"));
        } else {
            env::set_var(&("VAGGAENV_".to_string() + &k[..]),
                env::var_os(&k).unwrap_or(From::from("")));
        }
    }
    for pair in set_env.into_iter() {
        let mut pairiter = pair[..].splitn(2, '=');
        let key = "VAGGAENV_".to_string() + pairiter.next().unwrap();
        if let Some(value) = pairiter.next() {
            env::set_var(&key, value.to_string());
        } else {
            env::remove_var(&key);
        }
    }

    if settings.propagate_environ.len() > 0 {
        let regex = patterns_to_regex(&settings.propagate_environ)
            .context("can't compile propagate-environ patterns")?;
        for (key, value) in env::vars() {
            if regex.is_match(&key) {
                let key = "VAGGAENV_".to_string() + &key;
                if env::var_os(&key).is_some() {
                    continue;
                }
                env::set_var(key, value);
            }
        }
    }
    Ok(())
}
