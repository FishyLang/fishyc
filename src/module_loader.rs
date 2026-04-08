use std::collections::{ HashMap, HashSet };
use std::path::{ Path, PathBuf };
use crate::ast::{ Expr, Stmt };
use crate::token::Literal;
use crate::scanner::Scanner;
use crate::parser::Parser;

pub struct ModuleLoader {
    cache: HashMap<PathBuf, Vec<Stmt>>,
    loading_stack: HashSet<PathBuf>,
    pub sources: HashMap<String, String>,
    pub parse_errors: Vec<crate::parser::ParseError>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            loading_stack: HashSet::new(),
            sources: HashMap::new(),
            parse_errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, stmts: Vec<Stmt>, base_dir: &Path) -> Vec<Stmt> {
        let mut result = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::Using { names, source, keyword } => {
                    let path_str = match self.extract_path(&source) {
                        Some(p) => p,
                        None => {
                            self.parse_errors.push(crate::parser::ParseError {
                                token: keyword.clone(),
                                message: "'using' path must be a string literal.".to_string(),
                                hints: vec![],
                            });
                            continue;
                        }
                    };

                    let module_path = base_dir.join(&path_str);
                    let module_path = match module_path.canonicalize() {
                        Ok(p) => p,
                        Err(e) => {
                            self.parse_errors.push(crate::parser::ParseError {
                                token: keyword.clone(),
                                message: format!("Could not import module '{}': {}", path_str, e),
                                hints: vec![],
                            });
                            continue;
                        }
                    };

                    let imported_names: HashSet<String> = names
                        .iter()
                        .map(|t| t.lexeme.clone())
                        .collect();

                    let module_stmts = self.load_module(module_path);
                    result.extend(module_stmts.clone());

                    for decl in &module_stmts {
                        if let Stmt::Impl { target_type, .. } = decl {
                            let target_name = match target_type {
                                crate::ast::Type::Custom(name) => Some(name.clone()),
                                crate::ast::Type::Generic(name, _) => Some(name.clone()),
                                _ => None,
                            };
                            if let Some(tn) = target_name {
                                if imported_names.contains(&tn) {
                                    result.push(decl.clone());
                                }
                            }
                        }
                    }
                }

                other => result.push(other),
            }
        }

        result
    }

    pub fn load_module(&mut self, path: PathBuf) -> Vec<Stmt> {
        if let Some(cached) = self.cache.get(&path) {
            return cached.clone();
        }

        if self.loading_stack.contains(&path) {
            eprintln!(
                "Warning: Circular import detected for '{}'. Skipping to avoid infinite loop.",
                path.display()
            );
            return vec![];
        }
        self.loading_stack.insert(path.clone());

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path.display(), e);
                return vec![];
            }
        };

        let path_str = path.to_string_lossy().into_owned();
        self.sources.insert(path_str.clone(), source.clone());

        let mut scanner = Scanner::new(&source, &path_str);
        scanner.scan_tokens();

        for s_err in scanner.errors {
            self.parse_errors.push(crate::parser::ParseError {
                token: crate::token::Token::synthetic(crate::token::TokenType::Eof, "Scanner"),
                message: s_err.message,
                hints: vec![format!("Check file '{}'", path_str)],
            });
        }

        let mut parser = Parser::new(scanner.tokens);
        let stmts = parser.parse();

        self.parse_errors.extend(parser.errors);

        let module_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let resolved = self.resolve(stmts, &module_dir);
        self.loading_stack.remove(&path);
        self.cache.insert(path, resolved.clone());
        resolved
    }

    fn extract_path(&self, expr: &Expr) -> Option<String> {
        if let Expr::Literal(Literal::String(s)) = expr { Some(s.clone()) } else { None }
    }
}

fn decl_name(stmt: &Stmt) -> Option<String> {
    match stmt {
        Stmt::Function { name, .. } => Some(name.lexeme.clone()),
        Stmt::Struct { name, .. } => Some(name.lexeme.clone()),
        Stmt::Enum { name, .. } => Some(name.lexeme.clone()),
        Stmt::Trait { name, .. } => Some(name.lexeme.clone()),
        Stmt::ExternFunction { name, .. } => Some(name.lexeme.clone()),
        Stmt::Var { name, .. } => Some(name.lexeme.clone()),
        Stmt::Alias { name, .. } => Some(name.lexeme.clone()),
        _ => None,
    }
}
