use std::collections::HashMap;
use crate::token::Token;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[34m";

pub enum Severity {
    Error,
    Warning,
}

pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub label: String,
    pub hints: Vec<String>,
}

pub struct DiagnosticRenderer {
    pub files: HashMap<String, Vec<String>>,
}

impl DiagnosticRenderer {
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }

    pub fn add_file(&mut self, filename: String, source: &str) {
        self.files.insert(
            filename,
            source
                .lines()
                .map(|l| l.to_owned())
                .collect()
        );
    }

    pub fn emit(&self, diag: &Diagnostic) {
        let (sev_color, sev_text) = match diag.severity {
            Severity::Error => (RED, "Error"),
            Severity::Warning => (YELLOW, "Warning"),
        };

        eprintln!("{sev_color}{BOLD}{sev_text}{RESET}{BOLD}: {}{RESET}", diag.message);

        if diag.line == 0 || diag.file_path == "<internal>" {
            eprintln!();
            return;
        }

        let line_digits = format!("{}", diag.line).len();
        let pad = " ".repeat(line_digits);

        eprintln!("{BLUE}{pad} --> {RESET}{}:{}:{}", diag.file_path, diag.line, diag.col_start);
        eprintln!("{BLUE}{pad} |{RESET}");

        if let Some(lines) = self.files.get(&diag.file_path) {
            if let Some(source_line) = lines.get(diag.line - 1) {
                eprintln!(
                    "{BLUE}{:>width$} |{RESET} {}",
                    diag.line,
                    source_line,
                    width = line_digits
                );

                let spaces = " ".repeat(diag.col_start.saturating_sub(1));
                let caret_len = if diag.col_end >= diag.col_start {
                    diag.col_end - diag.col_start + 1
                } else {
                    1
                };
                let carets = "^".repeat(caret_len);

                if diag.label.is_empty() {
                    eprintln!("{BLUE}{pad} |{RESET} {spaces}{sev_color}{BOLD}{carets}{RESET}");
                } else {
                    eprintln!(
                        "{BLUE}{pad} |{RESET} {spaces}{sev_color}{BOLD}{carets} {}{RESET}",
                        diag.label
                    );
                }
            }
        }
        eprintln!("{BLUE}{pad} |{RESET}");

        for hint in &diag.hints {
            eprintln!("{BLUE}{pad} ={RESET} {BOLD}Hint:{RESET} {hint}");
        }
        eprintln!();
    }
}

pub fn span_from_token(token: &Token) -> (String, usize, usize, usize) {
    let col_end = token.column;
    
    let last_line_len = token.lexeme.chars().rev().take_while(|&c| c != '\n').count();
    let col_start = col_end.saturating_sub(last_line_len).saturating_add(1);
    
    ((*token.file_path).clone(), token.line, col_start, col_end)
}

pub fn make_error(token: &Token, message: impl Into<String>, hints: Vec<String>) -> Diagnostic {
    let (file_path, line, col_start, col_end) = span_from_token(token);
    Diagnostic {
        severity: Severity::Error,
        message: message.into(),
        file_path,
        line,
        col_start,
        col_end,
        label: String::new(),
        hints,
    }
}
