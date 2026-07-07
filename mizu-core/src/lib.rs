#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

/// Um comando simples (único, sem pipe).
#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub args: Vec<String>,
    pub stdin_file: Option<String>,
    pub stdout_file: Option<String>,
    pub stdout_append: bool,
}

/// Pipeline = sequência de comandos conectados por pipe.
#[derive(Debug, Clone)]
pub enum Pipeline {
    Single(SimpleCommand),
    Pipe(Box<Pipeline>, SimpleCommand),
}

/// Erro de parsing.
#[derive(Debug)]
pub enum ParseError {
    UnterminatedQuote,
    Empty,
}

/// Tokeniza uma linha de comando.
pub fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(core::mem::take(&mut current));
                }
            }
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }
            _ => current.push(c),
        }
    }

    if in_single || in_double {
        return Err(ParseError::UnterminatedQuote);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    if tokens.is_empty() {
        return Err(ParseError::Empty);
    }
    Ok(tokens)
}

/// Parseia tokens em um pipeline.
pub fn parse_pipeline(tokens: &[String]) -> Result<Pipeline, ParseError> {
    let mut pipe_groups: Vec<Vec<String>> = Vec::new();
    let mut current = Vec::new();

    for token in tokens {
        if token == "|" {
            if current.is_empty() {
                return Err(ParseError::Empty);
            }
            pipe_groups.push(core::mem::take(&mut current));
        } else {
            current.push(token.clone());
        }
    }
    if !current.is_empty() {
        pipe_groups.push(current);
    }

    let commands: Vec<SimpleCommand> = pipe_groups
        .into_iter()
        .map(|group| parse_simple(&group))
        .collect();

    if commands.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut pipeline = Pipeline::Single(commands[0].clone());
    for cmd in commands.into_iter().skip(1) {
        pipeline = Pipeline::Pipe(Box::new(pipeline), cmd);
    }
    Ok(pipeline)
}

fn parse_simple(tokens: &[String]) -> SimpleCommand {
    let mut args: Vec<String> = Vec::new();
    let mut stdin_file = None;
    let mut stdout_file = None;
    let mut stdout_append = false;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "<" => {
                i += 1;
                if i < tokens.len() {
                    stdin_file = Some(tokens[i].clone());
                }
            }
            ">" => {
                i += 1;
                if i < tokens.len() {
                    stdout_file = Some(tokens[i].clone());
                    stdout_append = false;
                }
            }
            ">>" => {
                i += 1;
                if i < tokens.len() {
                    stdout_file = Some(tokens[i].clone());
                    stdout_append = true;
                }
            }
            arg => args.push(String::from(arg)),
        }
        i += 1;
    }

    SimpleCommand { args, stdin_file, stdout_file, stdout_append }
}

/// Expande `~` no início de caminhos.
pub fn expand_path(path: &str, home: &str) -> String {
    if path == "~" {
        String::from(home)
    } else if path.starts_with("~/") {
        let mut s = String::from(home);
        s.push_str(&path[1..]);
        s
    } else {
        String::from(path)
    }
}

#[cfg(feature = "std")]
pub fn expand_path_env(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    expand_path(path, &home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let t = tokenize("ls -la /tmp").unwrap();
        assert_eq!(t, vec!["ls", "-la", "/tmp"]);
    }

    #[test]
    fn test_tokenize_quotes() {
        let t = tokenize(r#"echo "hello world" 'single quotes'"#).unwrap();
        assert_eq!(t, vec!["echo", "hello world", "single quotes"]);
    }

    #[test]
    fn test_tokenize_pipe() {
        let t = tokenize("ls | grep foo").unwrap();
        assert_eq!(t, vec!["ls", "|", "grep", "foo"]);
    }

    #[test]
    fn test_parse_redirect() {
        let tokens = tokenize("cat < input.txt > output.txt").unwrap();
        let cmd = parse_simple(&tokens);
        assert_eq!(cmd.args, vec!["cat"]);
        assert_eq!(cmd.stdin_file, Some("input.txt".into()));
        assert_eq!(cmd.stdout_file, Some("output.txt".into()));
    }

    #[test]
    fn test_parse_pipeline() {
        let tokens = tokenize("ls -la | grep mizu | wc -l").unwrap();
        let pipeline = parse_pipeline(&tokens).unwrap();
        match pipeline {
            Pipeline::Pipe(_, _) => assert!(true),
            _ => panic!("expected pipe"),
        }
    }

    #[test]
    fn test_expand_path() {
        assert_eq!(expand_path("~/foo", "/root"), "/root/foo");
        assert_eq!(expand_path("~", "/home/mizu"), "/home/mizu");
        assert_eq!(expand_path("/bin", "/root"), "/bin");
    }
}
