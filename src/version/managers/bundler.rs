use std::io::{Read, BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

use regex::Regex;
use shaman::digest::Digest;

use config::builders::GemBundleInfo;
use version::error::Error;

#[derive(Debug)]
enum ArgValue {
    Single(String),
    List(Vec<String>),
}

impl ArgValue {
    fn parse(input: &str) -> Self {
        let trim = |c| c == ' ' || c == ':';
        if input.starts_with("[") && input.ends_with("]") {
            ArgValue::List(input[1..(input.len()-1)]
            .split(",")
            .map(|x| x.trim_matches(|c| trim(c)).to_owned())
            .collect())
        } else {
            ArgValue::Single(input.trim_matches(|c| trim(c)).to_owned())
        }
    }
}

/// Hash Gemfile data,
///
/// Iterates over provided Gemfile contents, hashing each line that is a gem declaration.
pub fn hash(info: &GemBundleInfo, hash: &mut Digest)
    -> Result<(), Error>
{
    // Match a source line of the Gemfile
    let re_source = Regex::new(r"(?m)^source '(.+?)'").expect("Invalid regex");

    // Match a line describing a gem dependency in the format:
    //   gem 'gem-name', 'optional version', positional: :arguments
    let re_gem = Regex::new(
        r"(?xm)^\s*?gem\s
          '([:alpha:][\w-]*?)' # gem name
          (?:,\s*?'(.+?)')? # optional gem version
          (?:,\s*?(.*?))? # aditional info
          (?:\s*?\#.*?)?$ # ignore comments"
    ).expect("Invalid regex");

    // Match the start of a group block
    let re_group = Regex::new(r"(?m)^group\s+?(.*?)\s+?do(?:\s?#.*?)?$")
        .expect("Invalid regex");

    // Match the start of a path block
    let re_path = Regex::new(r"(?m)^path\s+?('.*?')\s+?do(?:\s?#.*?)?$")
        .expect("Invalid regex");

    let path = Path::new("/work").join(&info.gemfile);

    let gemlock = try!(path.parent()
        .map(|dir| dir.join("Gemfile.lock"))
        .ok_or("Gemfile should be under /work".to_owned()));
    if gemlock.exists() {
        try!(hash_lock_file(&gemlock, hash));
    }

    hash.input(b"-->\0");

    let mut f = try!(File::open(&path)
        .map_err(|e| Error::Io(e, path.clone())));
    let mut reader = BufReader::new(f);

    let mut lines = reader.lines();
    while let Some(line) = lines.next() {
        let line = try!(line.map_err(|e| Error::Io(e, path.to_path_buf())));
        let line = line.trim();
        // try to get source lines of Gemfile which usually contains:
        //   source 'https://rubygems.org'
        if let Some(cap) = re_source.captures(line) {
            hash.input(b"source");
            hash.input(cap[1].as_bytes());
        // try to match a gem declaration
        } else if let Some(cap) = re_gem.captures(line) {
            let caps: Vec<&str> = cap.iter().map(|c| c.unwrap_or("")).collect();
            try!(hash_gem_line(info, &caps, hash));
        // try to match the start of a group block
        } else if let Some(cap) = re_group.captures(line) {
            let groups = ArgValue::List(cap[1]
                .split(",")
                .map(|x| x.trim_matches(|c| c == ' ' || c == ':').to_owned())
                .collect());
            if should_skip(&groups, info) {
                try!(skip_group(&mut lines, &path));
            }
        // try to match the start of a path block
        } else if let Some(cap) = re_group.captures(line) {
            let path_block = &cap[1];
            try!(process_path_block(&mut lines, path_block, info, &re_path, hash, &path));
        }
    }

    hash.input(b"<--\0");
    Ok(())
}

/// Tell whether the group should be skipped
fn should_skip(groups: &ArgValue, info: &GemBundleInfo) -> bool {
    match groups {
        &ArgValue::Single(ref value) => {
            info.without.contains(&value.to_owned())
        }
        &ArgValue::List(ref list) => {
            list.iter()
            .fold(true, |acc, group| info.without.contains(group) && acc)
        }
    }
}

/// Skip a group block
///
/// Iterates over the lines, ignoring everything until a line containing "end" is found
fn skip_group<B>(lines: &mut Lines<B>, filename: &Path)
    -> Result<(), Error>
    where B: BufRead
{
    while let Some(line) = lines.next() {
        let line = try!(line.map_err(|e| Error::Io(e, filename.to_path_buf())));
        if line.trim() == "end" { break }
    }
    Ok(())
}

fn process_path_block<B>(lines: &mut Lines<B>,
                         path: &str,
                         info: &GemBundleInfo,
                         re: &Regex,
                         hash: &mut Digest,
                         filename: &Path)
                         -> Result<(), Error>
                         where B: BufRead
{
    while let Some(line) = lines.next() {
        let line = try!(line.map_err(|e| Error::Io(e, filename.to_path_buf())));
        if line.trim() == "end" { break }
        if let Some(cap) = re.captures(&line) {
            let mut caps: Vec<String> = cap.iter()
                .map(|c| c.unwrap_or("").to_owned())
                .collect();
            caps.get_mut(2).map(|args| {
                if args.len() > 0 {
                    args.push(',');
                }
                args.push_str(" :path => ");
                args.push_str(path);
            });
            let caps: Vec<_> = caps.iter().map(|c| c.as_ref()).collect();
            try!(hash_gem_line(info, &caps, hash));
        }
    }
    Ok(())
}

fn switch_in_list(input: &str, target: char, replacement: &str) -> String {
    let mut in_list = false;
    let replacer = |c| {
        if c == '[' && !in_list { in_list = true; }
        else if c == ']' && in_list { in_list = false; }
        return in_list && c == target
    };
    input.replace(replacer, replacement)
}

fn change_commas(input: &str) -> String {
    switch_in_list(input, ',', ";")
}

fn restore_commas(input: &str) -> String {
    switch_in_list(input, ';', ",")
}

fn parse_pos_args(cap: &str) -> Result<Vec<(String, ArgValue)>, String> {
    let re_keyword = Regex::new(r"(?:(.+?):\s+?(.+))").expect("Invalid regex");
    let re_arrow = Regex::new(r"(?::(.+?)\s+?=>\s+?(.+))").expect("Invalid regex");

    if !cap.is_empty() {
        // change ',' with ',' in lists '[]' so we can split args by ','
        let cap = change_commas(cap);
        let mut pos_args = Vec::new();
        for arg in cap.split(",") {
            let parts = try!(re_keyword.captures(&arg)
                .or(re_arrow.captures(&arg))
                .ok_or(format!("Invalid gem argument: {}", &arg)));

            let part_1 = parts[1].to_owned();
            // restore ','
            let part_2 = restore_commas(parts[2].trim_matches(':'));
            pos_args.push((part_1, ArgValue::parse(&part_2)));
        }
        Ok(pos_args)
    } else {
        Ok(Vec::new())
    }
}

/// Try to parse a gem dependency line
fn hash_gem_line(info: &GemBundleInfo, cap: &Vec<&str>, hash: &mut Digest)
    -> Result<(), String>
{
    let gem_name = cap[1];
    let gem_version = if cap[2] == "" { "*" } else { cap[2] };
    let pos_args = try!(parse_pos_args(cap[3]));

    // check if gem is in excluded group
    let skip_gem = pos_args.iter().position(|&(ref k, ref v)| {
            k == "group" &&
            should_skip(v, info)
        }).is_some();
    if skip_gem { return Ok(()) }

    hash.input(gem_name.as_bytes());
    hash.input(gem_version.as_bytes());
    for (key, value) in pos_args {
        hash.input(key.as_bytes());
        match value {
            ArgValue::Single(value) => hash.input(value.as_bytes()),
            ArgValue::List(list) => for i in list {
                hash.input(i.as_bytes());
            }
        }
    }

    hash.input(b"\0");
    Ok(())
}

fn hash_lock_file(path: &Path, hash: &mut Digest) -> Result<(), Error> {
    let mut f = try!(File::open(path).map_err(|e| Error::Io(e, path.to_path_buf())));
    let mut reader = BufReader::new(f);

    let re_gem = Regex::new(r"^\s*?(?P<gem>[\w-]+?)\s*?\((?P<version>[\w\.-]+?)\)?$")
        .expect("Invalid regex");

    let re_extra = Regex::new(r"^\s*?(?P<key>[\w-]+?):\s+?(?P<value>.+?)$")
        .expect("Invalid regex");

    hash.input(b"-->\0");
    let mut indent_size = 0;
    for line in reader.lines() {
        let line = try!(line.map_err(|e| Error::Io(e, path.to_path_buf())));

        // check first indented line for indent size
        if line.starts_with(" ") && indent_size == 0 {
            indent_size = find_indent(&line);
        }

        let indent_level = get_indent_level(&line, indent_size);

        if indent_level == 1 &&
            !line.trim().starts_with("specs:")
        {
            if let Some(cap) = re_extra.captures(&line) {
                hash.input(cap["key"].as_bytes());
                hash.input(cap["value"].as_bytes());
            }
            continue
        }

        if indent_level == 2 {
            if let Some(cap) = re_gem.captures(&line) {
                hash.input(cap["gem"].as_bytes());
                hash.input(cap["version"].as_bytes());
                hash.input(b"\0");
            }
        }
    }

    hash.input(b"<--\0");
    Ok(())
}

fn find_indent(line: &str) -> usize {
    let mut indent = 0;
    for c in line.chars() {
        if c != ' ' { break }
        indent += 1;
    }
    indent
}

fn get_indent_level(line: &str, indent: usize) -> usize {
    if indent == 0 { return 0 }
    let line_indent = find_indent(line);
    line_indent / indent
}
