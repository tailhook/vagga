use std::collections::HashSet;

use config::Config;

pub fn scan(config: &Config, src: Vec<String>) -> Vec<String> {
    let empty = Vec::new();
    return _scan(|name| {
        config.commands.get(name)
        .map(|x| x.prerequisites().iter())
        .unwrap_or(empty.iter())
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
}
