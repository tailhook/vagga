use argparse::{ArgumentParser, Collect};
use super::env::Environ;


pub fn env_options(env: &mut Environ, ap: &mut ArgumentParser) {
    ap.refer(&mut env.variables)
      .add_option(&["-v", "--variant"], box Collect::<String>,
            "Use variant where KEY equals VALUE (repeatable)")
      .metavar("KEY=VALUE");
}
