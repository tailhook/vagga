use std::io::{Read, BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

use regex::Regex;
use shaman::digest::Digest;

use config::builders::GemBundleInfo;
use version::error::Error;

struct GemlineHasher<'a> {
    info: &'a GemBundleInfo,
    re_gemline: Regex,
    re_group_keyword: Regex,
    re_group_arrow: Regex,
}

impl<'a> GemlineHasher<'a> {
    pub fn new(info: &'a GemBundleInfo) -> Self {
        // Match a line describing a gem dependency in the format:
        //   gem 'gem-name', 'optional version', positional: :arguments
        let re_gem = Regex::new(
            r"(?xm)^\s*?gem\s
              '([:alpha:][\w-]*?)' # gem name
              (?:,\s*?'(.+?)')? # optional gem version
              (?:,\s*?(.*?))? # extra args
              (?:\s*?\#.*?)?$ # ignore comments"
        ).expect("Invalid regex");

        let re_keyword = Regex::new(r"(?m),\s+?group:\s+?(.+?)(?:,\s+?\w+?:.*?)?$")
            .expect("Invalid regex");
        let re_arrow = Regex::new(r"(?m),\s+?:group\s+?=>\s+?(.+?)(?:,\s+?:\w+?\s+?=>\s+?.*?)?$")
            .expect("Invalid regex");

        GemlineHasher {
            info: info,
            re_gemline: re_gem,
            re_group_keyword: re_keyword,
            re_group_arrow: re_arrow,
        }
    }

    pub fn is_gemline(&self, line: &str) -> bool {
        self.re_gemline.is_match(line)
    }

    pub fn hash(&self, line: &str, hash: &mut Digest) {
        if let Some(captures) = self.re_gemline.captures(line) {
            let gem_name = &captures[1];
            let gem_version = captures.at(2).unwrap_or("*");
            let gem_extra = captures.at(3).unwrap_or("");
            // check if gem is in excluded group
            if let Some(cap) = self.re_group_keyword.captures(gem_extra) {
                if should_skip(&cap[1], self.info) {
                    return
                }
            }
            if let Some(cap) = self.re_group_arrow.captures(gem_extra) {
                if should_skip(&cap[1], self.info) {
                    return
                }
            }

            hash.input(gem_name.as_bytes());
            hash.input(gem_version.as_bytes());
            hash.input(gem_extra.as_bytes());

            hash.input(b"\0");
        }
    }
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

/// Hash Gemfile data,
///
/// Iterates over provided Gemfile contents, hashing each line that is a gem declaration.
pub fn hash(info: &GemBundleInfo, hash: &mut Digest)
    -> Result<(), Error>
{
    // Match a source line of the Gemfile
    let re_source = Regex::new(r"(?m)^source '(.+?)'").expect("Invalid regex");

    // Match the start of a group block
    let re_group = Regex::new(r"(?m)^group\s+?(.*?)\s+?do(?:\s?#.*?)?$")
        .expect("Invalid regex");

    // Match the start of a path block
    let re_path = Regex::new(r"(?m)^path\s+?('.*?')\s+?do(?:\s?#.*?)?$")
        .expect("Invalid regex");

    // Match the start of a source block
    let re_path = Regex::new(r"(?m)^source\s+?('.*?')\s+?do(?:\s?#.*?)?$")
        .expect("Invalid regex");

    let gemline_hasher = GemlineHasher::new(info);

    let path = Path::new("/work").join(&info.gemfile);

    let gemlock = try!(path.parent()
        .map(|dir| dir.join("Gemfile.lock"))
        .ok_or("Gemfile should be under /work".to_owned()));
    if gemlock.exists() {
        try!(hash_lock_file(&gemlock, hash));
    }

    hash.input(b"-->\0");

    let f = try!(File::open(&path).map_err(|e| Error::Io(e, path.clone())));
    let reader = BufReader::new(f);

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
        } else if gemline_hasher.is_gemline(line) {
            gemline_hasher.hash(line, hash);
        // try to match the start of a group block
        } else if let Some(cap) = re_group.captures(line) {
            let groups = &cap[1];
            if should_skip(&groups, info) {
                try!(skip_group(&mut lines, &path));
            }
        // try to match the start of a path block
        } else if let Some(cap) = re_path.captures(line) {
            let path_name = &cap[1];
            try!(process_block(&gemline_hasher, hash, &mut lines, path_name, &path));
        // try to match the start of a source block
        } else if let Some(cap) = re_path.captures(line) {
            let source_name = &cap[1];
            try!(process_block(&gemline_hasher, hash, &mut lines, source_name, &path));
        }
    }

    hash.input(b"<--\0");
    Ok(())
}

fn process_block<B>(gemline_hasher: &GemlineHasher,
                    hash: &mut Digest,
                    lines: &mut Lines<B>,
                    ident: &str,
                    filename: &Path)
                    -> Result<(), Error>
                    where B: BufRead
{
    while let Some(line) = lines.next() {
        let line = try!(line.map_err(|e| Error::Io(e, filename.to_path_buf())));
        if line.trim() == "end" { break }

        if gemline_hasher.is_gemline(&line) {
            hash.input(ident.as_bytes());
            gemline_hasher.hash(&line, hash);
        }
    }
    Ok(())
}

fn hash_lock_file(path: &Path, hash: &mut Digest) -> Result<(), Error> {
    let f = try!(File::open(path).map_err(|e| Error::Io(e, path.to_path_buf())));
    let reader = BufReader::new(f);

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
