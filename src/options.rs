use argparse::{ArgumentParser, Collect, StoreFalse};
use super::env::Environ;


pub fn env_options(env: &mut Environ, ap: &mut ArgumentParser) {
    ap.refer(&mut env.variables)
      .add_option(&["-v", "--variant"], box Collect::<String>,
            "Use variant where KEY equals VALUE (repeatable)")
      .metavar("KEY=VALUE");
    ap.refer(&mut env.settings.version_check)
      .add_option(&["--no-check"], box StoreFalse,
            "Do not check if container is up to date when running command");
}
