use std::str::Lines;

use regex::Regex;
use shaman::digest::Digest;
use config::builders::GemBundleInfo;

/// Hash Gemfile data,
///
/// Iterates over provided Gemfile contents, hashing each line that is a gem declaration.
pub fn hash(info: &GemBundleInfo, gemfile_contents: &str, hash: &mut Digest)
    -> Result<(), String>
{
    let re_source = Regex::new(r"(?m)^source '(.+?)'(?:\s?#.*?)?$")
    .expect("Invalid regex");
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
    let re_group = Regex::new(r"(?m)^group\s+?(.*?)\s+?do$").expect("Invalid regex");

    // Match a group block with gem declarations in the same line
    let re_group_inline = Regex::new(r"^group (.*?) do\s+?(.*?)\s+?end$").expect("Invalid regex");

    let mut lines = gemfile_contents.lines();
    while let Some(line) = lines.next() {
        let line = line.trim();
        // try to get source lines of Gemfile which usually contains:
        //   source 'https://rubygems.org'
        if let Some(cap) = re_source.captures(line) {
            hash.input(cap[1].as_bytes());
        // try to match a gem declaration
        } else if let Some(cap) = re_gem.captures(line) {
            let caps: Vec<&str> = cap.iter().map(|c| c.unwrap_or("")).collect();
            try!(hash_gem_line(info, &caps, hash));
        // try to match an inline group block
        } else if let Some(cap) = re_group_inline.captures(line) {
            if should_skip(&cap[1], info) { continue }
            for gem in cap[2].split(";") {
                let gem = gem.trim();
                // match the gem declaration found in the inline group block
                if let Some(cap) = re_gem.captures(gem) {
                    let caps: Vec<&str> = cap.iter().map(|c| c.unwrap_or("")).collect();
                    try!(hash_gem_line(info, &caps, hash));
                }
            }
        // try to match the start of a group block
        } else if let Some(cap) = re_group.captures(line) {
            if should_skip(&cap[1], info) {
                skip_group(&mut lines);
            }
        }
    }

    Ok(())
}

/// Tell whether the group should be skipped
fn should_skip(groups: &str, info: &GemBundleInfo) -> bool {
    groups.split(",")
        .map(|g| g.trim_matches(|c| [' ', ':', '[', ']'].contains(&c)))
        // need to alocate to satisfy type checker
        // &String != &str
        .fold(true, |acc, group| info.without.contains(&group.to_owned()) && acc)
}

/// Skip a group block
///
/// Iterates over the lines, ignoring everything until a line containing "end" is found
fn skip_group(lines: &mut Lines) {
    while let Some(line) = lines.next() {
        if line.trim() == "end" {
            break
        }
    }
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

fn switch_comma_with_pipe(input: &str) -> String {
    switch_in_list(input, ',', "|")
}

fn switch_pipe_with_comma(input: &str) -> String {
    switch_in_list(input, '|', ",")
}

fn parse_pos_args(cap: &str) -> Result<Vec<(String, String)>, String> {
    let re_keyword = Regex::new(r"(?:(.+?):\s+?(.+))").expect("Invalid regex");
    let re_arrow = Regex::new(r"(?::(.+?)\s+?=>\s+?(.+))").expect("Invalid regex");

    if !cap.is_empty() {
        let cap = switch_comma_with_pipe(cap);
        let mut pos_args = Vec::new();
        for arg in cap.split(",") {
            let parts = try!(re_keyword.captures(&arg)
                .or(re_arrow.captures(&arg))
                .ok_or(format!("Invalid gem argument: {}", &arg)));

            let part_1 = &parts[1];
            let part_2 = switch_pipe_with_comma(parts[2].trim_matches(':'));
            pos_args.push((part_1.to_owned(), part_2.to_owned()));
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
    // If we are here, at least the gem name was captured
    // let gem_name = cap.at(1).expect("Invalid regex capture"); // gem name
    // let gem_version = cap.at(2).unwrap_or("*"); // gem version
    // let pos_args = try!(parse_pos_args(&cap));

    let gem_name = cap[1];
    let gem_version = if cap[2] == "" { "*" } else { cap[2] };
    let pos_args = try!(parse_pos_args(cap[3]));

    // check if gem is in excluded group
    let skip_gem = pos_args.iter().position(|&(ref k, ref v)| {
            k == "group" &&
            should_skip(v, info)
            // need to alocate to satisfy type checker
            // &String != &str
            // info.without.contains(&v.to_owned())
        }).is_some();
    if skip_gem { return Ok(()) }

    hash.input(gem_name.as_bytes());
    hash.input(gem_version.as_bytes());
    for (key, value) in pos_args {
        hash.input(key.as_bytes());
        hash.input(value.as_bytes());
    }

    Ok(())
}
