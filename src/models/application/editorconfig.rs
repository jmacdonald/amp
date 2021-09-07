use core::ops::RangeInclusive;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::errors::*;
use regex::{self, Regex};

lazy_static! {
    static ref GLOB_SECTION_RE: Regex = Regex::new(r"^\[(.*)\]").unwrap();
    static ref KEY_VALUE_PAIR_RE: Regex = Regex::new(r"^(\w+)\s?=\s?([-\w\d]+)").unwrap();

    static ref GLOB_NUMERIC_RANGE_RE: Regex = Regex::new(r"^\{([+-]?\d+)\.\.([+-]?\d+)\}").unwrap();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndentSize {
    Tab,
    Width(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndOfLine {
    Lf,
    Cr,
    Crlf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Charset {
    Latin1,
    Utf8,
    Utf8Bom,
    Utf16Be,
    Utf16Le,
}

#[derive(Clone)]
pub struct Section {
    regex: Regex,

    pub indent_style: Option<IndentStyle>,
    pub indent_size: Option<IndentSize>,
    pub tab_width: Option<usize>,
    pub end_of_line: Option<EndOfLine>,
    pub charset: Option<Charset>,
    pub trim_trailing_whitespace: Option<bool>,
    pub insert_final_newline: Option<bool>,
}

impl Section {
    fn from(regex: Regex) -> Self {
        Self {
            regex,
            indent_style: None,
            indent_size: None,
            tab_width: None,
            end_of_line: None,
            charset: None,
            trim_trailing_whitespace: None,
            insert_final_newline: None,
        }
    }

    fn is_match(&self, filename: &str) -> bool {
        self.regex.is_match(filename)
    }

    fn insert_property(&mut self, name: &str, value: &str) {
        match &*name.to_ascii_lowercase() {
            "indent_style" => {
                self.indent_style = match &*value.to_ascii_lowercase() {
                    "tab" => Some(IndentStyle::Tab),
                    "space" => Some(IndentStyle::Space),
                    _ => None,
                };
            },

            "indent_size" => {
                self.indent_size =
                    if &*value.to_ascii_lowercase() == "tab" {
                        Some(IndentSize::Tab)
                    } else if let Ok(value) = value.parse::<usize>() {
                        Some(IndentSize::Width(value))
                    } else {
                        None
                    };
            },

            "tab_width" => {
                self.tab_width = value.parse::<usize>().ok();
            },

            "end_of_line" => {
                self.end_of_line = match &*value.to_ascii_lowercase() {
                    "lf" => Some(EndOfLine::Lf),
                    "cr" => Some(EndOfLine::Cr),
                    "crlf" => Some(EndOfLine::Crlf),
                    _ => None,
                };
            },

            "charset" => {
                self.charset = match &*value.to_ascii_lowercase() {
                    "latin1" => Some(Charset::Latin1),
                    "utf-8" => Some(Charset::Utf8),
                    "utf-8-bom" => Some(Charset::Utf8Bom),
                    "utf-16be" => Some(Charset::Utf16Be),
                    "utf-16le" => Some(Charset::Utf16Le),
                    _ => None,
                };
            },

            "trim_trailing_whitespace" => {
                self.trim_trailing_whitespace = match &*value.to_ascii_lowercase() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                };
            },

            "insert_final_newline" => {
                self.insert_final_newline = match &*value.to_ascii_lowercase() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                };
            },

            _ => {},
        };
    }
}

pub struct EditorConfig {
    sections: Vec<Section>,
}

impl EditorConfig {
    pub fn from_directory(mut path: PathBuf) -> Result<Option<EditorConfig>> {
        path.push(".editorconfig");

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Ok(None),
        };

        load_from_file(&file)
            .map(|sections| Some(EditorConfig { sections }))
            .chain_err(|| "Failed to parse .editorconfig file")
    }

    pub fn get(&self, filename: &str) -> Option<&Section> {
        for section in &self.sections {
            if section.is_match(filename) {
                return Some(section);
            }
        }

        None
    }
}

fn load_from_file(file: &File) -> Result<Vec<Section>> {
    let reader = BufReader::new(file);

    let mut last_glob = None;
    let mut glob_list = vec![];
    let mut line_num = 1;

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };

        let line = line.trim();

        // Ignore empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if let Some(capture) = GLOB_SECTION_RE.captures(&line) {
            if let Some(glob) = last_glob {
                glob_list.push(glob);
            }

            if let Some(regex) = glob_to_regex(&capture[1]).and_then(|r| Regex::new(&r).ok()) {
                last_glob = Some(Section::from(regex));
            } else {
                last_glob = None;
            }
        } else if let Some(capture) = KEY_VALUE_PAIR_RE.captures(&line) {
            if let Some(ref mut glob) = last_glob {
                glob.insert_property(&capture[1], &capture[2]);
            }
        } else {
            return Err(format!("parse failure on line {}", line_num).into());
        }

        line_num += 1;
    }

    if let Some(glob) = last_glob {
        glob_list.push(glob);
    }

    // We reverse the list here to preserve the property that always the last
    // value that was specified for that property for a specific glob match
    // will be used.
    glob_list.reverse();
    Ok(glob_list)
}

fn glob_to_regex(glob: &str) -> Option<String> {
    Some(glob_to_regex_inner(glob)? + "$")
}

fn glob_to_regex_inner(glob: &str) -> Option<String> {
    //
    // EditorConfig specifies only a small amount of wildcard patterns:
    //
    //   | glob pattern | description
    //   |--------------|-------------
    //   | *            | matches any string of characters except /
    //   | **           | matches any string of characters
    //   | ?            | matches any single character except /
    //   | [name]       | matches any single character between [ and ], no nesting allowed
    //   | [!name]      | matches any single character _not_ between [ and ], no nesting allowed
    //   | {a,b,c}      | matches any of the comma-separated strings between { and }, nesting allowed
    //   | {num1..num2} | matches any integer number between num1 and num2 (inclusive), nesting allowed
    //

    let mut result = Vec::new();
    let mut next_index = 0;

    for (index, pattern) in glob.match_indices(&['\\', '*', '?', '[', '{', '/'][..]) {
        if index < next_index {
            continue;
        }

        // Add anything between last and current match to our result
        result.push(regex::escape(&glob[next_index..index]));

        match pattern {
            "\\" => {
                if let Some(next) = glob.get(index+1..=index+1) {
                    result.push(next.to_string());
                    next_index = index + 2;
                } else { // We're at the end of the glob
                    result.push(r"\\".to_string());
                    next_index = index + 1;
                }
            },

            "?" => {
                result.push(r"[^/]".to_string());
                next_index = index + 1;
            },

            "*" => {
                if index < glob.len()-1 && &glob[index+1..=index+1] == "*" {
                    result.push(".*".to_string());
                    next_index = index + 2;
                } else {
                    result.push(r"[^/]*".to_string());
                    next_index = index + 1;
                }
            },

            "[" => {
                let consumed = parse_brackets(&glob[index..], &mut result)?;
                next_index = index + consumed;
            },

            "{" => {
                let consumed = parse_braces(&glob[index..], &mut result)?;
                next_index = index + consumed;
            },

            "/" => {
                if index + 3 < glob.len() && &glob[index..=index+3] == "/**/" {
                    result.push(r"(/|/.*/)".to_string());
                    next_index = index + 4;
                } else {
                    result.push("/".to_string());
                    next_index = index + 1;
                }
            },

            #[cfg(debug_assertions)]
            _ => unreachable!(),

            #[cfg(not(debug_assertions))]
            _ => return None,
        }
    }

    result.push(regex::escape(&glob[next_index..]));
    Some(result.join(""))
}

fn parse_brackets(glob: &str, result: &mut Vec<String>) -> Option<usize> {
    let end_index = find_end_index(glob, '[', ']')?;

    // Ignore empty brackets
    if end_index == 1 {
        return Some(2);
    } else if end_index == 2 && &glob[1..=1] == "!" {
        return Some(3);
    }

    if glob.get(1..=1)? == "!" {
        result.push("[^".to_string());
        result.push(regex::escape(&glob[2..end_index]));
    } else {
        result.push("[".to_string());
        result.push(regex::escape(&glob[1..end_index]));
    }

    result.push("]".to_string());
    Some(end_index + 1)
}

fn parse_braces(glob: &str, result: &mut Vec<String>) -> Option<usize> {
    if let Some(cap) = GLOB_NUMERIC_RANGE_RE.captures(glob) {
        result.push(create_regex_range(&cap[1], &cap[2])?);
        Some(cap.get(0)?.end())
    } else {
        let end_index = find_end_index(glob, '{', '}')?;

        let glob = &glob[1..end_index];
        if glob.len() == 0 {
            return Some(2);
        }

        let mut start_index = 0;
        result.push("(".to_string()); // open group
        for (index, _) in glob.match_indices(',') {
            if index > 0 && &glob[index-1..=index-1] == "\\" {
                continue;
            }

            if index - start_index > 0 {
                let glob = &glob[start_index..index];
                let nesting_level = parse_nesting_level(glob);

                if nesting_level == 0 {
                    result.push("(".to_string());
                    result.push(glob_to_regex_inner(glob)?);
                    result.push(")|".to_string());
                } else {
                    continue;
                }
            } else {
                result.push("$|".to_string());
            }

            start_index = index + 1;
        }

        result.push("(".to_string());
        result.push(glob_to_regex_inner(&glob[start_index..])?);
        result.push(")".to_string());

        result.push(")".to_string()); // close group
        Some(end_index + 1)
    }
}

fn create_regex_range(start: &str, end: &str) -> Option<String> {
    let mut range = create_range(start, end)?
        .into_iter()
        .fold(String::new(), |acc, n| acc + &n.to_string() + "|");

    // Remove last pipe
    range.pop();

    Some(format!("({})", range))
}

fn create_range(start: &str, end: &str) -> Option<RangeInclusive<i32>> {
    let start = start.parse::<i32>().ok()?;
    let end = end.parse::<i32>().ok()?;

    if start <= end {
        Some(start..=end)
    } else {
        Some(end..=start)
    }
}

fn find_end_index(input: &str, open: char, close: char) -> Option<usize> {
    let mut level = 0;
    let mut last_char = None;
    let mut last_close_index = None;

    for (index, c) in input.char_indices() {
        if c == open {
            if last_char.is_none() || (last_char.is_some() && last_char.unwrap() != '\\') {
                level += 1;
            }
        } else if c == close {
            if last_char.is_none() || (last_char.is_some() && last_char.unwrap() != '\\') {
                last_close_index = Some(index);
                level -= 1;
            }
        }

        if level == 0 {
            return Some(index);
        }

        last_char = Some(c);
    }

    last_close_index
}

fn parse_nesting_level(input: &str) -> i32 {
    let mut level = 0;
    let mut last_char = None;

    for c in input.chars() {
        if c == '{' || c == '[' {
            if last_char.is_none() || (last_char.is_some() && last_char.unwrap() != '\\') {
                level += 1;
            }
        } else if c == '}' || c == ']' {
            if last_char.is_none() || (last_char.is_some() && last_char.unwrap() != '\\') {
                level -= 1;
            }
        }

        last_char = Some(c);
    }

    level
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    macro_rules! glob_lit_matches {
        ($glob:literal, $expanded:literal) => {
            assert_eq!(glob_to_regex($glob).unwrap(), $expanded);
        };
    }

    macro_rules! glob_matches {
        ($glob:literal, $path:literal) => {
            assert!(Regex::new(&glob_to_regex($glob).unwrap()).unwrap().is_match($path));
        };
        ($glob:literal, $path: literal, $($rest:literal),+) => {
            glob_matches!($glob, $path);
            glob_matches!($glob, $($rest),+);
        };
    }

    macro_rules! glob_matches_not {
        ($glob:literal, $path:literal) => {
            assert!(!Regex::new(&glob_to_regex($glob).unwrap()).unwrap().is_match($path));
        };
        ($glob:literal, $path: literal, $($rest:literal),+) => {
            glob_matches_not!($glob, $path);
            glob_matches_not!($glob, $($rest),+);
        };
    }

    #[test]
    fn project_editorconfig() {
        let config = EditorConfig::from_directory(
            Path::new(env!("CARGO_MANIFEST_DIR"))
        ).unwrap();

        let md = config.get("README.md").unwrap();
        assert_eq!(md.regex.as_str(), r"[^/]*\.md$");

        let all = config.get("src/main.rs").unwrap();
        assert_eq!(all.regex.as_str(), r"[^/]*$");

        assert!(md.is_match("README.md"));
        assert!(all.is_match("README.md"));
        assert!(all.is_match("src/models/application/editorconfig.rs"));

        assert!(!md.trim_trailing_whitespace.unwrap());

        assert!(all.insert_final_newline.unwrap());
        assert!(all.trim_trailing_whitespace.unwrap());
        assert_eq!(all.end_of_line.unwrap(), EndOfLine::Lf);
        assert_eq!(all.charset.unwrap(), Charset::Utf8);
        assert_eq!(all.indent_style.unwrap(), IndentStyle::Space);
        assert_eq!(all.indent_size.unwrap(), IndentSize::Width(4));
    }

    #[test]
    fn glob_range() {
        assert_eq!(create_regex_range("0", "3"), Some("(0|1|2|3)".into()));
        assert_eq!(create_regex_range("0", "0"), Some("(0)".into()));
        assert_eq!(create_regex_range("4", "2"), Some("(2|3|4)".into()));
        assert_eq!(create_regex_range("-4", "-2"), Some("(-4|-3|-2)".into()));
        assert_eq!(create_regex_range("-2", "2"), Some("(-2|-1|0|1|2)".into()));
    }

    #[test]
    fn single_star_globs() {
        glob_lit_matches!("*", r"[^/]*$");
        glob_matches!("*", "src/main.rs", "README.md");

        glob_lit_matches!("*.rs", r"[^/]*\.rs$");
        glob_matches_not!("*.rs", "README.md", "src/input/key_map/default.yml");

        glob_lit_matches!("a*/z", "a[^/]*/z$");
        glob_matches!("a*/z", "a/z", "ab/z", "abcdefg/z");
        glob_matches_not!("a*/z", "a/b/z");
    }

    #[test]
    fn double_star_globs() {
        glob_lit_matches!("**/*.rs", r".*/[^/]*\.rs$");
        glob_matches!("**/*.rs", "src/main.rs");
        glob_matches_not!("**/*.rs", "README.md");

        glob_lit_matches!("src/**/mod.rs", r"src(/|/.*/)mod\.rs$");
        glob_matches!("src/**/mod.rs", "src/commands/mod.rs", "src/models/application/mod.rs");
        glob_matches_not!("src/**/mod.rs", "src/commands/buffer.rs", "src/view/style.rs");
    }

    #[test]
    fn question_mark_globs() {
        glob_lit_matches!("a?c/d", r"a[^/]c/d$");
        glob_matches!("a?c/d", "abc/d");
        glob_matches_not!("a?c/d", "a/c/d", "ac/d");
    }

    #[test]
    fn braces_number_ranges_globs() {
        glob_lit_matches!("a/t{1..3}/b", "a/t(1|2|3)/b$");
        glob_matches!("a/t{1..3}/b", "a/t1/b", "a/t3/b");
        glob_matches_not!("a/t{1..3}/b", "a/t/b/", "a/t0/b", "a/t4/b");

        glob_lit_matches!("a/t{-20..-17}/b", "a/t(-20|-19|-18|-17)/b$");
        glob_matches!("a/t{-20..-17}/b", "a/t-20/b", "a/t-17/b");
        glob_matches_not!("a/t{-20..-17}/b", "a/t-21/b", "a/t-16/b");

        glob_lit_matches!("a/t{-2..1}/b", "a/t(-2|-1|0|1)/b$");
        glob_matches!("a/t{-2..1}/b", "a/t-2/b", "a/t0/b", "a/t1/b");
    }

    #[test]
    fn braces_any_globs() {
        glob_lit_matches!("*.{rs,md,yml}", r"[^/]*\.((rs)|(md)|(yml))$");
        glob_matches!("*.{rs,md,yml}", "src/main.rs", "README.md", "src/input/key_map/default.yml");
        glob_matches_not!("*.{rs,md,yml}", ".gitmodules", "src/main.c");

        glob_lit_matches!("*{.rs,.md,.yml}", r"[^/]*((\.rs)|(\.md)|(\.yml))$");
        glob_matches!("*{.rs,.md,.yml}", "src/main.rs", "README.md", "src/input/key_map/default.yml");
        glob_matches_not!("*{.rs,.md,.yml}", ".gitmodules", "src/main.c");

        glob_lit_matches!("*.{r\\,s,m\\,d,ym\\,l}", r"[^/]*\.((r,s)|(m,d)|(ym,l))$");
        glob_matches!("*.{r\\,s,m\\,d,ym\\,l}", "src/main.r,s", "README.m,d", "src/input/key_map/default.ym,l");
        glob_matches_not!("*.{r\\,s,m\\,d,ym\\,l}", "src/main.rs", "README.md", "src/input/key_map/default.yml");

        glob_lit_matches!("*.{,rs,md,yml}", r"[^/]*\.($|(rs)|(md)|(yml))$");
        glob_matches!("*.{,rs,md,yml}", "src/main.rs", "README.md", "src/input/key_map/default.yml", "test.");
        glob_matches_not!("*.{,rs,md,yml}", "test", ".gitmodules");

        glob_lit_matches!("*.{}", r"[^/]*\.$");
        glob_matches!("*.{}", "test.");
        glob_matches_not!("*.{}", "src/main.rs", "README.md", ".gitmodules");

        glob_lit_matches!("src/ma{}in.rs", r"src/main\.rs$");
        glob_matches!("src/ma{}in.rs", "src/main.rs");
    }

    #[test]
    fn nested_braces_any_globs() {
        glob_lit_matches!("x{a,b,z[123]}", "x((a)|(b)|(z[123]))$");
        glob_matches!("x{a,b,z[123]}", "xa", "xb", "xz1", "xz3");
        glob_matches_not!("x{a,b,z[123]}", "xc", "x1", "x3", "xa1", "xb1", "x");

        glob_lit_matches!("x{a,b{1,2,3}}", "x((a)|(b((1)|(2)|(3))))$");
        glob_matches!("x{a,b{1,2,3}}", "xa", "xb1", "xb2", "xb3");
        glob_matches_not!("x{a,b{1,2,3}}", "x", "xb", "xb4", "xba", "xab");

        glob_lit_matches!("x{a,{1,2,3}}", "x((a)|(((1)|(2)|(3))))$");
        glob_matches!("x{a,{1,2,3}}", "xa", "x1", "x2", "x3");
        glob_matches_not!("x{a,{1,2,3}}", "x", "xb", "xa1", "xba", "xab");

        glob_lit_matches!("x{a,b{1,2,3},z[!123]}", "x((a)|(b((1)|(2)|(3)))|(z[^123]))$");
        glob_matches!("x{a,b{1,2,3},z[!123]}", "xa", "xb1", "xb2", "xb3", "xz4", "xzA");
        glob_matches_not!("x{a,b{1,2,3},z[!123]}", "xc", "xb", "xz", "xa1", "xab", "xz1", "xz3");

        glob_lit_matches!("x{1,{2..4},5}", "x((1)|((2|3|4))|(5))$");
        glob_matches!("x{1,{2..4},5}", "x1", "x2", "x3", "x4", "x5");
        glob_matches_not!("x{1,{2..4},5}", "x0", "x6");
    }

    #[test]
    fn brackets_globs() {
        glob_lit_matches!("[abcdef].rs", r"[abcdef]\.rs$");
        glob_matches!("[abcdef].rs", "a.rs", "c.rs", "f.rs");
        glob_matches_not!("[abcdef].rs", "s.rs", "q.rs", "y.rs");

        glob_lit_matches!("a[c]e.rs", r"a[c]e\.rs$");
        glob_matches!("a[c]e.rs", "ace.rs");
        glob_matches_not!("a[c]e.rs", "abe.rs");

        glob_lit_matches!("a[]e.rs", r"ae\.rs$");
        glob_matches!("a[]e.rs", "ae.rs");
        glob_matches_not!("a[]e.rs", "ace.rs");

        glob_lit_matches!("a[b[c].rs", r"a[b\[c]\.rs$");
        glob_matches!("a[b[c].rs", "ab.rs", "ac.rs", "a[.rs");
        glob_matches_not!("a[b[c].rs", "abc.rs", "ab[c.rs");
    }

    #[test]
    fn brackets_not_globs() {
        glob_lit_matches!("[!abcdef].rs", r"[^abcdef]\.rs$");
        glob_matches!("[!abcdef].rs", "s.rs", "q.rs", "y.rs");
        glob_matches_not!("[!abcdef].rs", "a.rs", "c.rs", "f.rs");

        glob_lit_matches!("a[!c]e.rs", r"a[^c]e\.rs$");
        glob_matches!("a[!c]e.rs", "abe.rs");
        glob_matches_not!("a[!c]e.rs", "ace.rs");

        glob_lit_matches!("a[!]e.rs", r"ae\.rs$");
        glob_matches!("a[!]e.rs", "ae.rs");
        glob_matches_not!("a[!]e.rs", "ace.rs");
    }

    #[test]
    fn nested_brackets_globs() {
        glob_lit_matches!("[ab[cd]ef].rs", r"[ab\[cd\]ef]\.rs$");
        glob_matches!("[ab[cd]ef].rs", "a.rs", "b.rs", "[.rs", "c.rs", "d.rs", "].rs", "e.rs", "f.rs");
        glob_matches_not!("[ab[cd]ef].rs", "g.rs", ".rs");

        glob_lit_matches!("[ab[cd]ef].rs", r"[ab\[cd\]ef]\.rs$");
        glob_matches!("[ab[cd]ef].rs", "a.rs", "b.rs", "[.rs", "c.rs", "d.rs", "].rs", "e.rs", "f.rs");
        glob_matches_not!("[ab[cd]ef].rs", "g.rs", ".rs");
    }
}
