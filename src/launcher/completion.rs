use config::{Config};


pub fn generate_completions(config: &Config, args: Vec<String>) -> Result<i32, String> {
    let mut completions: Vec<&str> = vec!();
    let default_cur_arg = "".to_string();
    let mut splitted_args = args.rsplitn(2, |a| a == "--");
    let cur_arg = match splitted_args.next() {
        Some(a) => a.get(0).unwrap_or(&default_cur_arg),
        None => &default_cur_arg,
    };
    let full_args = match splitted_args.next() {
        Some(a) => a.iter().collect::<Vec<_>>(),
        None => vec!(),
    };

    if full_args.len() == 0 {
        let builtin_commands = vec!(
            "_build",
            "_build_shell",
            "_clean",
            "_create_netns",
            "_destroy_netns",
            "_init_storage_dir",
            "_list",
            "_pack_image",
            "_run",
            "_run_in_netns",
            "_version_hash",
            );
        for (cmd, _) in &config.commands {
            completions.push(cmd);
        }
        if cur_arg.starts_with("_") {
            for cmd in builtin_commands {
                completions.push(cmd);
            }
        }
    } else if full_args.len() == 1 {
        let builtin_commands_with_container = vec!(
            "_build",
            "_build_shell",
            "_pack_image",
            "_run",
            "_run_in_netns",
            "_version_hash",
            );
        if let Some(cmd) = full_args.get(0) {
            if builtin_commands_with_container.iter().any(|c| c == cmd) {
                for (container, _) in &config.containers {
                    completions.push(container);
                }
            }
        }
    }

    for comp in &completions {
        if comp.starts_with(cur_arg) {
            println!("{}", comp);
        }
    }

    Ok(0)
}
