extern crate globset;
extern crate regex;
extern crate walkdir;
#[macro_use] extern crate quick_error;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use globset::{Error as GlobError, GlobBuilder, GlobSet, GlobSetBuilder};
use regex::{Error as RegexError, Regex};
use walkdir::{DirEntry, Error as WalkDirError, IntoIter as WalkDirIter};
use walkdir::{WalkDir};

quick_error! {
    #[derive(Debug)]
    pub enum FilterError {
        Regex(err: RegexError) {
            description("regex error")
            display("regex error: {}", err)
            from()
        }
        Glob(err: GlobError) {
            description("globset error")
            display("globset error: {}", err)
            from()
        }
        Utf8(path: PathBuf) {
            description("invalid utf-8 path")
            display("path is not utf-8: {:?}", path)
        }
        WalkDir(err: WalkDirError) {
            description("walk dir error")
            display("walk dir error: {}", err)
            from()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Rule {
    orig: String,
    glob: String,
    is_ignore: bool,
    is_absolute: bool,
    is_dir: bool,
    is_exact: bool,
    is_intermediate: bool,
    literal_sep: bool,
}

#[derive(Debug, PartialEq)]
pub enum Match<'a> {
    Include(Option<&'a Rule>),
    Ignore(Option<&'a Rule>),
    None,
}

impl<'a> Match<'a> {
    pub fn is_include(&self) -> bool {
        if let Match::Include(_) = *self {
            true
        } else {
            false
        }
    }
    pub fn is_ignore(&self) -> bool {
        if let Match::Ignore(_) = *self {
            true
        } else {
            false
        }
    }
    pub fn is_none(&self) -> bool {
        Match::None == *self
    }
}

#[derive(Debug)]
pub enum PathFilter {
    Glob {
        globs: GlobSet,
        rules: Vec<Rc<Rule>>,
        skip_unknown_dirs: bool,
    },
    Re {
        ignore: Option<Regex>,
        include: Option<Regex>,
    },
}

impl PathFilter {
    pub fn glob<S: AsRef<str>>(rules: &[S])
        -> Result<PathFilter, FilterError>
    {
        let mut globset = GlobSetBuilder::new();
        let mut ruleset = vec!();
        let mut skip_unknown_dirs = true;
        let mut prepared_rules = vec!();
        let mut unique_rules = HashSet::new();
        for rule_str in rules.iter().rev() {
            prepare_rule(rule_str.as_ref(), &mut prepared_rules,
                &mut unique_rules);
            while let Some(rule) = prepared_rules.pop() {
                if !rule.is_absolute && !rule.is_ignore {
                    skip_unknown_dirs = false;
                }
                ruleset.push(rule);
            }
        }
        ruleset.reverse();
        for rule in ruleset.iter() {
            globset.add(GlobBuilder::new(&rule.glob)
                        .literal_separator(rule.literal_sep)
                        .build()?);
        }

        Ok(PathFilter::Glob {
            globs: globset.build()?,
            rules: ruleset,
            skip_unknown_dirs: skip_unknown_dirs,
        })
    }

    pub fn regex<R1, R2>(ignore: Option<R1>, include: Option<R2>)
        -> Result<PathFilter, FilterError>
        where R1: AsRef<str>, R2: AsRef<str>
    {
        Ok(PathFilter::Re {
            ignore: if let Some(s) = ignore {
                Some(Regex::new(s.as_ref())?)
            } else {
                None
            },
            include: if let Some(s) = include {
                Some(Regex::new(s.as_ref())?)
            } else {
                None
            },
        })
    }

    pub fn matched<S: AsRef<str>>(&self, path: S, is_dir: bool) -> Match {
        let path = path.as_ref();
        match *self {
            PathFilter::Glob {globs: ref globset, ref rules, ..} => {
                let mut matched_rule_ix = None;
                for &glob_ix in globset.matches(path).iter().rev() {
                    let rule = &rules[glob_ix];
                    if is_dir || !rule.is_dir {
                        if matched_rule_ix.is_none() || rule.is_exact {
                            matched_rule_ix = Some(glob_ix);
                        }
                        if rule.is_exact {
                            break;
                        }
                    }
                }
                if let Some(rule_ix) = matched_rule_ix {
                    let rule = &rules[rule_ix];
                    if rule.is_ignore {
                        Match::Ignore(Some(rule))
                    } else {
                        Match::Include(Some(rule))
                    }
                } else {
                    Match::None
                }
            },
            PathFilter::Re {ref ignore, ref include} => {
                if let Some(ref ignore_re) = *ignore {
                    if ignore_re.is_match(path) {
                        return Match::Ignore(None);
                    }
                }
                if let Some(ref include_re) = *include {
                    if include_re.is_match(path) {
                        return Match::Include(None);
                    }
                }
                Match::None
            }
        }
    }

    pub fn walk<P, R, F>(&self, path: P, f: F)
        -> Result<R, Vec<FilterError>>
        where P: AsRef<Path>, R: Sized, F: FnOnce(Walker) -> R
    {
        let mut errors = vec!();
        let res = {
            let walker = self.walk_iterator(path, &mut errors);
            f(walker)
        };
        if errors.is_empty() {
            Ok(res)
        } else {
            Err(errors)
        }
    }

    pub fn walk_iterator<'a, 'b, P: AsRef<Path>>(&'a self, path: P,
        errors: &'b mut Vec<FilterError>)
        -> Walker<'a, 'b>
    {
        let root = path.as_ref();
        let walk_dir = WalkDir::new(root);
        Walker::new(root.to_path_buf(), walk_dir.into_iter(), self, errors)
    }
}

fn prepare_rule(rule: &str, cooked_rules: &mut Vec<Rc<Rule>>,
    unique_rules: &mut HashSet<Rc<Rule>>)
{
    let orig_rule = &rule[..];
    let mut rule = &rule[..];
    let is_ignore = if rule.starts_with('!') {
        rule = &rule[1..];
        true
    } else {
        false
    };
    if rule.starts_with('\\') {
        rule = &rule[1..];
    }
    let has_slash = rule.find('/').is_some();
    let is_dir = rule.ends_with('/');
    let is_absolute = rule.starts_with('/');
    if rule.ends_with('/') {
        rule = &rule[..rule.len() - 1];
    }
    if rule.starts_with('/') {
        rule = &rule[1..];
    }

    let prefix = if !is_absolute && !rule.starts_with("**/") {
        "**/"
    } else {
        ""
    };
    if is_ignore {
        let parsed_rule = Rc::new(Rule {
            orig: orig_rule.to_string(),
            glob: format!("{}{}", prefix, rule),
            is_ignore: is_ignore,
            is_absolute: is_absolute,
            is_dir: is_dir,
            is_exact: true,
            is_intermediate: false,
            literal_sep: has_slash,
        });
        maybe_add_rule(parsed_rule, cooked_rules, unique_rules);
    } else {
        // generate intermediate rules so afterwards we can enter into
        // these directories but do not include them
        let mut cur_glob = String::new();
        for part in rule.split('/') {
            cur_glob.push_str(part);
            if cur_glob == "" {
                continue;
            }
            let is_last = cur_glob.len() == rule.len();
            let parsed_rule = Rc::new(Rule {
                orig: orig_rule.to_string(),
                glob: format!("{}{}", prefix, cur_glob),
                is_ignore: is_ignore,
                is_absolute: is_absolute,
                is_dir: !is_last || (is_last && is_dir),
                is_exact: true,
                is_intermediate: !is_last,
                literal_sep: has_slash,
            });
            maybe_add_rule(parsed_rule, cooked_rules, unique_rules);
            cur_glob.push('/');
        }
    }
    if is_dir {
        let glob = if rule == "" {
            // it is root directory ("/") so we will match all
            format!("**/*")
        } else {
            // match all inside a directory but not the directory itself
            format!("{}{}/**/*", prefix, rule)
        };
        let parsed_rule = Rc::new(Rule {
            orig: orig_rule.to_string(),
            glob: glob,
            is_ignore: is_ignore,
            is_absolute: is_absolute,
            is_dir: false,
            is_exact: false,
            is_intermediate: false,
            literal_sep: has_slash,
        });
        maybe_add_rule(parsed_rule, cooked_rules, unique_rules);
    } else if !rule.ends_with("/**") {
        // match all nested files
        let parsed_rule = Rc::new(Rule {
            orig: orig_rule.to_string(),
            glob: format!("{}{}/**", prefix, rule),
            is_ignore: is_ignore,
            is_absolute: is_absolute,
            is_dir: false,
            is_exact: false,
            is_intermediate: false,
            literal_sep: has_slash,
        });
        maybe_add_rule(parsed_rule, cooked_rules, unique_rules);
    }
}

fn maybe_add_rule(rule: Rc<Rule>, rules: &mut Vec<Rc<Rule>>,
    unique_rules: &mut HashSet<Rc<Rule>>)
{
    if !unique_rules.contains(&rule) {
        unique_rules.insert(rule.clone());
        rules.push(rule.clone());
    }
}

pub struct Walker<'a, 'b> {
    root: PathBuf,
    dir_iter: WalkDirIter,
    filter: &'a PathFilter,
    errors: &'b mut Vec<FilterError>,
}

impl<'a, 'b> Walker<'a, 'b> {
    pub fn new(root: PathBuf, dir_iter: WalkDirIter, filter: &'a PathFilter,
        errors: &'b mut Vec<FilterError>)
        -> Walker<'a, 'b>
    {
        Walker {
            root: root,
            dir_iter: dir_iter,
            filter: filter,
            errors: errors,
        }
    }
}

impl<'a, 'b> Iterator for Walker<'a, 'b> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.dir_iter.next() {
                Some(Ok(ref entry)) => {
                    let path = entry.path();
                    let path = if let Ok(p) = path.strip_prefix(&self.root) {
                        p
                    } else {
                        path
                    };
                    if path == Path::new("") {
                        continue;
                    }
                    let entry_type = entry.file_type();
                    let path_str = if let Some(s) = path.to_str() {
                        s
                    } else {
                        self.errors.push(
                            FilterError::Utf8(path.to_path_buf()));
                        continue;
                    };
                    let matched = self.filter.matched(path_str, entry_type.is_dir());
                    match matched {
                        Match::Include(Some(ref rule))
                            if rule.is_intermediate ||
                            (entry_type.is_dir() && !rule.is_dir) =>
                        {
                            continue;
                        },
                        Match::Include(_) => {
                            return Some(entry.clone());
                        }
                        Match::Ignore(_) => {
                            if entry_type.is_dir() {
                                self.dir_iter.skip_current_dir();
                            }
                            continue;
                        },
                        Match::None => {
                            match *self.filter {
                                PathFilter::Glob {skip_unknown_dirs, ..} =>
                                {
                                    if entry_type.is_dir() && skip_unknown_dirs
                                    {
                                        self.dir_iter.skip_current_dir();
                                    }
                                    continue;
                                },
                                PathFilter::Re {ref include, ..} => {
                                    if include.is_some() {
                                        continue;
                                    }
                                    return Some(entry.clone());
                                }
                            }
                        },
                    }
                },
                Some(Err(e)) => {
                    self.errors.push(FilterError::from(e));
                },
                None => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::PathFilter;

    #[test]
    fn test_glob_relative_no_wildcards() {
        let f = PathFilter::glob(&vec!("foo")).unwrap();
        assert!(f.matched("foo", false).is_include());
        assert!(f.matched("foo", true).is_include());
        assert!(f.matched("test", false).is_none());
        assert!(f.matched("test", true).is_none());
        assert!(f.matched("dir/foo", false).is_include());
        assert!(f.matched("dir/foo", true).is_include());
        assert!(f.matched("foo/test", false).is_include());
        assert!(f.matched("foo/test", true).is_include());
        assert!(f.matched("dir/foo/test", false).is_include());
        assert!(f.matched("dir/foo/test", true).is_include());
    }

    #[test]
    fn test_glob_relative_with_wildcard() {
        let f = PathFilter::glob(&vec!("*.rs")).unwrap();
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/subdir/main.rs", false).is_include());
        assert!(f.matched("test.py", false).is_none());
        assert!(f.matched("test.rs.bak", false).is_none());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.py", false).is_none());
        assert!(f.matched(".git", false).is_none());
        assert!(f.matched(".git", true).is_none());
        assert!(f.matched(".git/test.rs", false).is_include());
    }

    #[test]
    fn test_glob_absolute_no_wildcards() {
        let f = PathFilter::glob(&vec!("/foo")).unwrap();
        assert!(f.matched("foo", false).is_include());
        assert!(f.matched("foo", true).is_include());
        assert!(f.matched("test", false).is_none());
        assert!(f.matched("test", true).is_none());
        assert!(f.matched("dir/foo", false).is_none());
        assert!(f.matched("dir/foo", true).is_none());
        assert!(f.matched("foo/test", false).is_include());
        assert!(f.matched("foo/test", true).is_include());
        assert!(f.matched("dir/foo/test", false).is_none());
        assert!(f.matched("dir/foo/test", true).is_none());
    }

    #[test]
    fn test_glob_absolute_with_wildcard() {
        let f = PathFilter::glob(&vec!("/*.rs")).unwrap();
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("test.py", false).is_none());
        assert!(f.matched("dir/test.rs", false).is_none());
    }

    #[test]
    fn test_glob_relative_dir() {
        let f = PathFilter::glob(&vec!("dir/")).unwrap();
        assert!(f.matched("test.rs", false).is_none());
        assert!(f.matched("otherdir/test.rs", false).is_none());
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("otherdir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_include());
        assert!(f.matched("dir/subdir", true).is_include());
        assert!(f.matched("dir/subdir/test.rs", false).is_include());
        assert!(f.matched("otherdir/dir", true).is_include());
        assert!(f.matched("otherdir/dir", false).is_none());
        assert!(f.matched("otherdir/dir/test.py", false).is_include());
        assert!(f.matched("otherdir/dir/subdir", true).is_include());
    }

    #[test]
    fn test_glob_absolute_dir() {
        let f = PathFilter::glob(&vec!("/dir/")).unwrap();
        assert!(f.matched("test.rs", false).is_none());
        assert!(f.matched("otherdir/test.rs", false).is_none());
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("otherdir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_include());
        assert!(f.matched("dir/subdir", true).is_include());
        assert!(f.matched("dir/subdir/test.rs", false).is_include());
        assert!(f.matched("otherdir/dir", true).is_none());
        assert!(f.matched("otherdir/dir", false).is_none());
        assert!(f.matched("otherdir/dir/test.py", false).is_none());
        assert!(f.matched("otherdir/dir/subdir", true).is_none());
    }

    #[test]
    fn test_glob_inverse() {
        let f = PathFilter::glob(&vec!("!.git/", "*.rs")).unwrap();
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/subdir/main.rs", false).is_include());
        assert!(f.matched("test.py", false).is_none());
        assert!(f.matched("test.rs.bak", false).is_none());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.py", false).is_none());
        assert!(f.matched(".git", false).is_none());
        assert!(f.matched(".git", true).is_ignore());
        assert!(f.matched(".git/test.rs", false).is_include());
    }

    #[test]
    fn test_glob_inverse_nested() {
        let f = PathFilter::glob(&vec!("!target/build", "*.rs")).unwrap();
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("target/test.rs", false).is_include());
        assert!(f.matched("target", false).is_none());
        assert!(f.matched("target", true).is_none());
        assert!(f.matched("target/build", false).is_ignore());
        assert!(f.matched("target/build", true).is_ignore());
    }

    #[test]
    fn test_glob_ignore_pattern_and_include_file() {
        let f = PathFilter::glob(&vec!("!*.py", "/dir")).unwrap();
        assert!(f.matched("test.rs", true).is_none());
        assert!(f.matched("test.py", true).is_ignore());
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_ignore());
    }

    #[test]
    fn test_glob_ignore_pattern_and_include_dir() {
        let f = PathFilter::glob(&vec!("!*.py", "/dir/")).unwrap();
        assert!(f.matched("test.rs", true).is_none());
        assert!(f.matched("test.py", true).is_ignore());
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_ignore());
    }

    #[test]
    fn test_glob_escape() {
        let f = PathFilter::glob(&vec!("\\!dir/")).unwrap();
        assert!(f.matched("!dir", false).is_none());
        assert!(f.matched("!dir", true).is_include());
        assert!(f.matched("!dir/test.rs", false).is_include());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("dir", true).is_none());

        let f = PathFilter::glob(&vec!("\\\\dir/")).unwrap();
        assert!(f.matched("\\dir", false).is_none());
        assert!(f.matched("\\dir", true).is_include());
        assert!(f.matched("\\dir/test.rs", false).is_include());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("dir", true).is_none());
    }

    #[test]
    fn test_glob_precedence() {
        let f = PathFilter::glob(&vec!("!.git/", ".git/")).unwrap();
        assert!(f.matched(".git", true).is_include());
        assert!(f.matched(".git", false).is_none());
    }

    #[test]
    fn test_glob_leading_double_asterisks() {
        let f = PathFilter::glob(&vec!("**/foo")).unwrap();
        assert!(f.matched("foo", true).is_include());
        assert!(f.matched("foo", false).is_include());
        assert!(f.matched("foo/test.rs", false).is_include());
        assert!(f.matched("foo/subdir", true).is_include());
        assert!(f.matched("foo/subdir/test.rs", false).is_include());
    }

    #[test]
    fn test_glob_intermediate_double_asterisks() {
        let f = PathFilter::glob(&vec!("dir/**/foo")).unwrap();
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir", false).is_none());
        assert!(f.matched("dir/test.rs", false).is_none());
        assert!(f.matched("dir/subdir", true).is_include());
        assert!(f.matched("dir/foo", false).is_include());
        assert!(f.matched("dir/foo", true).is_include());
        assert!(f.matched("dir/foo/test.rs", false).is_include());
        assert!(f.matched("dir/subdir/foo", false).is_include());
        assert!(f.matched("dir/subdir/foo", true).is_include());
        assert!(f.matched("dir/subdir/foo/test.rs", false).is_include());
    }

    #[test]
    fn test_glob_trailing_double_asterisks() {
        let f = PathFilter::glob(&vec!("dir/**")).unwrap();
        assert!(f.matched("dir", true).is_include());
        assert!(f.matched("dir", false).is_include());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/subdir", true).is_include());
        assert!(f.matched("dir/subdir/test.rs", false).is_include());
    }

    #[test]
    fn test_glob_unique_rules() {
        let f = PathFilter::glob(&vec!("/dir/test/", "/dir/test/")).unwrap();
        match f {
            PathFilter::Glob {ref rules, ..} => {
                let globs = rules.iter()
                    .map(|r| &r.glob)
                    .collect::<Vec<_>>();
                assert_eq!(globs, &["dir", "dir/test", "dir/test/**/*"]);
            },
            PathFilter::Re {..} => {
                panic!("Expected glob filter");
            },
        }
    }

    #[test]
    fn test_regex_empty() {
        let f = PathFilter::regex(None::<&str>, None::<&str>).unwrap();
        assert!(f.matched("test.ini", false).is_none());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_none());
        assert!(f.matched("dir/subdir/test.py", false).is_none());
    }


    #[test]
    fn test_regex_ignore() {
        let f = PathFilter::regex(
                Some(r"(^|/)\.(git|hg)($|/)|\.bak$|\.orig$"), None::<&str>)
            .unwrap();
        assert!(f.matched("test.ini", false).is_none());
        assert!(f.matched("test.bak", false).is_ignore());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_none());
        assert!(f.matched("dir/test.bak", false).is_ignore());
        assert!(f.matched("dir/subdir/module.orig", false).is_ignore());
        assert!(f.matched(".git", true).is_ignore());
        assert!(f.matched(".git", false).is_ignore());
        assert!(f.matched(".hg", true).is_ignore());
        assert!(f.matched(".svn", true).is_none());
        assert!(f.matched(".git/test.rs", false).is_ignore());
        assert!(f.matched(".git/objects", true).is_ignore());
        assert!(f.matched(".hg/test.ini", false).is_ignore());
    }

    #[test]
    fn test_regex_include() {
        let f = PathFilter::regex(None::<&str>, Some(r"(^|/).*\.rs$")).unwrap();
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("test.py", false).is_none());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_none());
        assert!(f.matched("dir/subdir", true).is_none());
        assert!(f.matched("dir/subdir/test.rs", false).is_include());
    }

    #[test]
    fn test_regex_ignore_and_include() {
        let f = PathFilter::regex(
                Some(r"(^|/)\.(git|hg)($|/)|\.bak$|\.orig$"),
                Some(r"(^|/).*(\.rs|\.ini)$"))
            .unwrap();
        assert!(f.matched("test.ini", false).is_include());
        assert!(f.matched("test.rs", false).is_include());
        assert!(f.matched("test.py", false).is_none());
        assert!(f.matched("dir", true).is_none());
        assert!(f.matched("dir/test.rs", false).is_include());
        assert!(f.matched("dir/test.py", false).is_none());
        assert!(f.matched("dir/test.bak", false).is_ignore());
        assert!(f.matched("dir/subdir/test.rs", false).is_include());
        assert!(f.matched("dir/subdir/test.py", false).is_none());
        assert!(f.matched("dir/subdir/test.bak", false).is_ignore());
        assert!(f.matched(".git", true).is_ignore());
        assert!(f.matched(".git", false).is_ignore());
        assert!(f.matched(".hg", true).is_ignore());
        assert!(f.matched(".svn", true).is_none());
        assert!(f.matched(".git/test.rs", false).is_ignore());
        assert!(f.matched(".git/objects", true).is_ignore());
        assert!(f.matched(".hg/test.ini", false).is_ignore());
    }
}
