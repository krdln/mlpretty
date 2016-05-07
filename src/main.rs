extern crate regex;
extern crate colored;

mod charpeek;

use charpeek::Charpeek;

use regex::Regex;
use colored::Colorize;

use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::io;
use std::iter::{once,repeat};
use std::env;
use std::process;

fn color_keywords(keys: &Regex, mut line: &str) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    while let Some((start, end)) = keys.find(line) {
        write!(buf, "{}{}", &line[..start], line[start..end].bold()).unwrap();
        line = &line[end..];
    }
    write!(buf, "{}", line).unwrap();
    buf
}

fn main() {
    let args: Vec<_> = env::args_os().collect();
    if args.len() <= 1 {
        let i = io::stdin();
        do_it(&mut i.lock());
    } else {
        let mut cmd = process::Command::new(&args[1])
            .args(&args[2..])
            .stdout(process::Stdio::piped())
            .spawn()
            .expect("Can't spawn process");
        do_it(&mut io::BufReader::new(cmd.stdout.as_mut().unwrap()));
        let status = cmd.wait().expect("Can't wait for process");
        if !status.success() {
            println!("{}: {}", "Exit status".yellow(), status);
        }
    }
}

fn do_it(reader: &mut BufRead) {
    let location_r = Regex::new(r#"^File "(.*)", line (\d+), characters (\d+)-(\d+):$"#).unwrap();
    let command_r = Regex::new(r"ocaml.* -I .* -o .*|^ocamldep.*\.mli? >|^ocamlbuild -package").unwrap();
    let val_r = Regex::new(r"^((?:let |val )?)(.+?) : (.+?) =(.*)$").unwrap();
    let type_r = Regex::new(r"^type (.+?) =(.*)$").unwrap();
    let word_r = Regex::new(r"^\w+").unwrap();
    let keywords_r = Regex::new(r"\b(let|in|match|with|for|do|done|if|then|else|begin|end|rec|when|and|or|val)\b").unwrap();
    let escape_r = Regex::new(r"\033\[.+?m").unwrap();

    let make_fence = |len: usize| repeat('-').take(len).collect::<String>();

    let mut lines = Charpeek::new(reader);
    let mut files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut inside = false;

    while let Some(byte) = lines.peek_byte() {
        // Handle prompt (we want to flush it immediately)
        if byte == b'#' || (byte as char).is_whitespace() {
            if byte == b'#' {
                print!("\n{}", "λ".green().bold());
                lines.flush_peek(io::sink());
            } else {
                lines.flush_peek(&mut io::stdout());
            }
            continue;
        }

        let line = if let Some(line) = lines.next_line() { line } else { break };
        let unescaped_line = escape_r.replace_all(&line, "");

        if let Some(captures) = location_r.captures(&unescaped_line) {
            // Handle error/warning
            inside = true;

            let filename = &captures[1];
            let i_row: usize = captures[2].parse().unwrap();
            let col_start: usize = captures[3].parse().unwrap();
            let mut col_end: usize = captures[4].parse().unwrap();

            let file = files.entry(filename.to_owned())
                .or_insert_with(||
                    fs::File::open(&filename)
                        .map(|file|
                             io::BufReader::new(file)
                                .lines()
                                .map(|x| x.unwrap_or_else(|_| "".into()))
                                .chain(once("<END OF FILE>".into()))
                                .collect()
                        )
                        .unwrap_or_else(|_| vec![])
                );

            let filerow = format!("{}:{}", filename, i_row);
            let fence = format!("-- {} {}", filerow, make_fence(80 - 4 - filerow.len()));
            println!("\n{}", fence.cyan());

            // The message
            let message = lines.next_line().unwrap_or("".into());
            let message = escape_r.replace_all(&message, "");
            let message_type = &message[..word_r.find(&message).unwrap_or((0,0)).1];
            let colorize = |s: &str| match message_type {
                "Warning" => s.yellow(),
                "Error" => s.red().bold(),
                _ => s.bold(),
            };
            println!("{}{}", colorize(message_type), &message[message_type.len()..]);
            while lines
                    .peek_byte()
                    .map_or(false, |b| (b as char).is_whitespace()) {
                println!("{}", lines.next_line().unwrap());
            }
            println!("");

            // The snippet
            for i in i_row.. {
                let rownum = format!("{}|", i);
                let row = &file.get(i - 1).map_or("<NO SUCH LINE>", |s| s.as_str());

                println!("{}{}", rownum.cyan(), color_keywords(&keywords_r, row));

                let col_start =
                    if i == i_row { col_start }
                    else { row.chars().take_while(|c| c.is_whitespace()).count() };

                for _ in 0..rownum.len() { print!(" ") }
                // print!("{}", "|".cyan());
                for c in row[..col_start].chars() {
                    if c.is_whitespace() { print!("{}", c) }
                    else { print!(" ") }
                }

                let underline_len = std::cmp::min(col_end, row.len()) - col_start;
                println!("{}", colorize(&repeat('~').take(underline_len).collect::<String>()));

                if col_end <= row.len() { break }
                else { col_end -= row.len() + 1 }
            }

        } else if command_r.is_match(&line) {
            // Handle commands printed by ocamlbuild
            if inside {
                println!("");
                inside = false;
            }
            println!("{}", line.dimmed());

        } else if let Some(captures) = val_r.captures(&line) {
            // Handle val/let assignments
            if inside {
                println!("");
                inside = false;
            }
            println!("{}{} : {} ={}", captures[1].bold(), &captures[2], captures[3].cyan(), &captures[4]);

        } else if let Some(captures) = type_r.captures(&line) {
            // Handle type declarations
            if inside {
                println!("");
                inside = false;
            }
            println!("{} {} ={}", "type".bold(), &captures[1].cyan(), &captures[2]);

        } else if line.starts_with("Hint:") {
            println!("\n{}\n{}{}", make_fence(80).cyan(), "Hint".green(), &line[4..]);

        } else {
            println!("{}", line);
        }
    }
}
