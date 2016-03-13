use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::str::Lines;

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
pub fn hash(info: &GemBundleInfo,hash: &mut Digest)
    -> Result<(), Error>
{
    let path = Path::new("/work").join(&info.gemfile);

    let gemfile_contents = try!(File::open(&path)
        .and_then(|mut f| {
            let mut buf = String::new();
            try!(f.read_to_string(&mut buf));
            Ok(buf)
        })
        .map_err(|e| Error::Io(e, path.clone()))
    );

    // Match a source line of the Gemfile
    let re_source = Regex::new(r"(?m)^source '(.+?)'").expect("Invalid regex");

    // Match a ruby version line
    let re_ruby = Regex::new(r"(?m)^ruby '(.+?)'").expect("Invalid regex");

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

    let mut lines = gemfile_contents.lines();
    while let Some(line) = lines.next() {
        let line = line.trim();
        // try to get source lines of Gemfile which usually contains:
        //   source 'https://rubygems.org'
        if let Some(cap) = re_source.captures(line) {
            hash.input(b"source");
            hash.input(cap[1].as_bytes());
        // try to match a ruby version line
        } else if let Some(cap) = re_ruby.captures(line) {
            hash.input(b"ruby");
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
                skip_group(&mut lines);
            }
        // try to match the start of a path block
        } else if let Some(cap) = re_group.captures(line) {
            let path = &cap[1];
            try!(process_path_block(&mut lines, path, info, &re_path, hash));
        }
    }

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
fn skip_group(lines: &mut Lines) {
    while let Some(line) = lines.next() {
        if line.trim() == "end" { break }
    }
}

fn process_path_block(lines: &mut Lines, path: &str,
                      info: &GemBundleInfo, re: &Regex,
                      hash: &mut Digest)
                      -> Result<(), String>
{
    while let Some(line) = lines.next() {
        if line.trim() == "end" { break }
        if let Some(cap) = re.captures(line) {
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

    Ok(())
}
