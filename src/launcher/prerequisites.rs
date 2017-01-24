use std::collections::HashSet;

use config::volumes::Volume;
use config::volumes::PersistentInfo;
use config::command::MainCommand;
use launcher::Context;


fn _check_volumes<'x, I>(ctx: &Context, iter: I, result: &mut Vec<&'x String>)
     where I: IntoIterator<Item=&'x Volume>
{
    for vol in iter.into_iter() {
        match vol {
            &Volume::Persistent(PersistentInfo {
                init_command: Some(ref cmd), ref name, .. })
            => {
                let path = if ctx.ext_settings.storage_dir.is_some() {
                    ctx.config_dir.join(".vagga/.lnk/.volumes").join(name)
                } else {
                    ctx.config_dir.join(".vagga/.volumes").join(name)
                };
                if !path.exists() {
                    result.push(&cmd);
                }
            }
            _ => {}
        }
    }
}

pub fn scan(ctx: &Context, src: Vec<String>) -> Vec<String> {
    return _scan(|name| {
        let mut all = Vec::new();
        if let Some(cmd) = ctx.config.commands.get(name) {
            match cmd {
                &MainCommand::Command(ref cmd) => {
                    all.extend(cmd.prerequisites.iter());
                    _check_volumes(ctx, cmd.volumes.values(), &mut all);
                    ctx.config.containers.get(&cmd.container)
                    .map(|cont|
                        _check_volumes(ctx, cont.volumes.values(), &mut all));
                }
                &MainCommand::CapsuleCommand(_) => {
                    // Should be nothing
                },
                &MainCommand::Supervise(ref sup) => {
                    all.extend(sup.prerequisites.iter());
                    for cmd in sup.children.values() {
                        all.extend(cmd.prerequisites().iter());
                        _check_volumes(ctx,
                            cmd.get_volumes().values(), &mut all);
                        ctx.config.containers.get(cmd.get_container())
                        .map(|cont| _check_volumes(ctx,
                            cont.volumes.values(), &mut all));
                    }
                }
            }
        }
        return all.into_iter();
    }, src);
}

fn _scan<'a, G, I>(get: G, src: Vec<String>) -> Vec<String>
    where G: Fn(&String) -> I, I: Iterator<Item=&'a String>
{
    // `Visited` protects from duplicates on adding to the result list
    let mut visited = src.iter().collect::<HashSet<_>>();
    // `Stackset` protects from cycles when traversing the stack
    let mut stackset = src.iter().collect::<HashSet<_>>();
    // In both cases we ensure that toplevel commands are run in the order
    // user specified, regardless of prerequisites
    let mut result = Vec::new();
    for toplevel in src.iter() {
        let mut stack = Vec::new();
        stack.push((toplevel, get(toplevel)));
        stackset.insert(toplevel);
        while let Some((name, mut iter)) = stack.pop() {
            match iter.next() {
                Some(cname) => {
                    stack.push((name, iter));
                    if stackset.contains(cname) {
                        continue;
                    }
                    stack.push((cname, get(cname)));
                    stackset.insert(cname);
                }
                None => {
                    stackset.remove(name);
                    if !visited.contains(name) {
                        visited.insert(name);
                        result.push(name.clone());
                    }
                    continue;
                }
            }
        }
        result.push(toplevel.clone());
    }
    return result;
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::_scan;

    fn do_scan(hash: &[(&str, &[&str])], src: &[&str], dest: &[&str]) {
        let pre: HashMap<String, Vec<String>> = hash.into_iter()
            .map(|&(k, v)| (String::from(k),
                            v.iter().map(|&x| String::from(x)).collect()))
            .collect::<HashMap<_, _>>();
        let commands = src.iter().map(|&x| String::from(x)).collect();
        let empty = Vec::new();
        assert_eq!(_scan(|name| {
                pre.get(name).map(|x| x.iter()).unwrap_or(empty.iter())
            }, commands).iter().map(|x| &x[..]).collect::<Vec<_>>(),
            dest);
    }

    #[test]
    fn test_no_cmds() {
        do_scan(&[], &["a", "b"], &["a", "b"]);
    }

    #[test]
    fn test_empty_pre() {
        do_scan(&[
            ("a", &[]),
            ("b", &[]),
        ], &["a", "b"], &["a", "b"]);
    }

    #[test]
    fn test_one() {
        do_scan(&[
            ("a", &["pa1"]),
            ("b", &["pb1", "pb2"]),
        ],
        &["a"],
        &["pa1", "a"]);
    }

    #[test]
    fn test_two() {
        do_scan(&[
            ("a", &["pa1", "pa2"]),
            ("b", &["pb1", "pb2"]),
        ],
        &["a", "b"],
        &["pa1", "pa2", "a", "pb1", "pb2", "b"]);
    }

    #[test]
    fn test_cycle_direct() {
        do_scan(&[("a", &["b"]), ("b", &["a"])], &["a", "b"], &["a", "b"]);
        do_scan(&[("a", &["b"]), ("b", &["a"])], &["b", "a"], &["b", "a"]);
        do_scan(&[("a", &["b"]), ("b", &["a"])], &["a"], &["b", "a"]);
        do_scan(&[("a", &["b"]), ("b", &["a"])], &["b"], &["a", "b"]);
    }

    #[test]
    fn test_cycle_deeper() {
        do_scan(&[
            ("a", &["b"]),
            ("b", &["a"]),
            ("c", &["b"]),
        ], &["c"], &["a", "b", "c"]);
    }

    #[test]
    fn test_common() {
        do_scan(&[
            ("a", &[]),
            ("b", &["a"]),
            ("c", &["a"]),
            ("d", &["b", "c"]),
        ],
        &["d"],
        &["a", "b", "c", "d"]);
    }
    #[test]
    fn test_multiple() {
        do_scan(&[
            ("a", &[]),
            ("b", &["a"]),
            ("c", &["a"]),
            ("d", &["b", "c"]),
        ],
        &["d", "d", "d"],
        &["a", "b", "c", "d", "d", "d"]);
    }

    #[test]
    fn test_override() {
        do_scan(&[
            ("d", &["b", "a", "a", "b"]),
        ],
        &["d"],
        &["b", "a", "d"]);
    }
}
