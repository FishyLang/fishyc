use crate::ast::{ Expr, Stmt };
use crate::module_names::Names;
use crate::parser::{ ParseError, Parser };
use crate::scanner::Scanner;
use crate::token::{ Literal, Token, TokenType };
use std::collections::{ HashMap, HashSet };
use std::path::{ Path, PathBuf };

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Private,
    Exported,
}
#[derive(Clone)]
struct Declaration {
    symbol: String,
    visibility: Visibility,
}

/// Modules keep their complete compilation contents. Import bindings refer to
/// declaration identities; selection never removes private implementation code.
pub struct ModuleLoader {
    declarations: HashMap<PathBuf, HashMap<String, Declaration>>,
    loading: HashSet<PathBuf>,
    emitted: HashSet<PathBuf>,
    next_module: usize,
    prelude: HashSet<String>,
    pub symbols: HashSet<String>,
    pub external_names: HashMap<String, String>,
    pub sources: HashMap<String, String>,
    pub parse_errors: Vec<ParseError>,
}
impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            loading: HashSet::new(),
            emitted: HashSet::new(),
            next_module: 1,
            prelude: HashSet::new(),
            symbols: HashSet::new(),
            external_names: HashMap::new(),
            sources: HashMap::new(),
            parse_errors: vec![],
        }
    }
    fn error(&mut self, token: &Token, message: impl Into<String>) {
        self.parse_errors.push(ParseError {
            token: token.clone(),
            message: message.into(),
            hints: vec![],
        });
    }
    pub fn resolve_root(
        &mut self,
        stmts: Vec<Stmt>,
        base: &Path,
        mut prelude: Vec<Stmt>
    ) -> Vec<Stmt> {
        self.prelude = prelude
            .iter()
            .filter(|s|
                matches!(
                    s,
                    Stmt::Export { .. } |
                        Stmt::Function { is_public: true, .. } |
                        Stmt::ExternFunction { .. }
                )
            )
            .filter_map(decl_name)
            .map(|n| n.lexeme.clone())
            .collect();
        // The monolithic prelude has the same public/private declaration syntax
        // as ordinary modules. Lower its export wrappers before type checking;
        // retain original private identities for its implementation bodies.
        prelude = prelude
            .into_iter()
            .map(|s| {
                match s {
                    Stmt::Export { declaration, .. } => *declaration,
                    s => s,
                }
            })
            .collect();
        // Primitive names can be receivers of standard-library static methods.
        self.prelude.extend(
            [
                "bool",
                "string",
                "u8",
                "u16",
                "u32",
                "u64",
                "i8",
                "i16",
                "i32",
                "i64",
                "f16",
                "f32",
                "f64",
            ]
                .into_iter()
                .map(str::to_string)
        );
        let root = stmts
            .iter()
            .find_map(stmt_token)
            .and_then(|t| Path::new(t.file_path.as_str()).canonicalize().ok());
        let root = root.unwrap_or_else(|| base.join("<root>"));
        self.emitted.insert(root.clone());
        prelude.extend(self.resolve_unit(stmts, &root, base, 0));
        prelude
    }
    fn resolve_unit(&mut self, stmts: Vec<Stmt>, path: &Path, base: &Path, id: usize) -> Vec<Stmt> {
        let mut own = HashMap::new();
        for stmt in &stmts {
            if let Some(name) = decl_name(stmt) {
                if name.lexeme.starts_with("__fishy_") || name.lexeme == "__global_init" {
                    self.error(name, "Name is reserved for the compiler runtime.");
                }
                let exported = matches!(
                    stmt,
                    Stmt::Export { .. } |
                        Stmt::Function {
                            is_public: true,
                            ..
                        }
                );
                let symbol = if id == 0 {
                    name.lexeme.clone()
                } else {
                    format!("__fishy_module_{id}_{}", name.lexeme)
                };
                self.symbols.insert(symbol.clone());
                let entry = own.entry(name.lexeme.clone()).or_insert(Declaration {
                    symbol: symbol.clone(),
                    visibility: Visibility::Private,
                });
                if exported {
                    entry.visibility = Visibility::Exported;
                }
                let inner = if let Stmt::Export { declaration, .. } = stmt {
                    declaration.as_ref()
                } else {
                    stmt
                };
                if matches!(inner, Stmt::ExternFunction { .. } | Stmt::Function { body: None, .. }) {
                    self.external_names.insert(symbol, name.lexeme.clone());
                }
            }
        }
        self.declarations.insert(path.to_path_buf(), own.clone());
        self.loading.insert(path.to_path_buf());
        let mut bindings = own
            .iter()
            .map(|(n, d)| (n.clone(), d.symbol.clone()))
            .collect::<HashMap<_, _>>();
        let mut pending = HashMap::new();
        for stmt in &stmts {
            if let Stmt::Using { keyword, names, source } = stmt {
                let Expr::Literal(Literal::String(source)) = source else {
                    self.error(keyword, "'using' path must be a string literal.");
                    continue;
                };
                let imported = match base.join(source).canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        self.error(keyword, format!("Could not import module '{source}': {e}"));
                        continue;
                    }
                };
                let contents = self.load_module(imported.clone());
                pending.insert(keyword.id, contents);
                if let Some(exports) = self.declarations.get(&imported).cloned() {
                    for name in names {
                        match exports.get(&name.lexeme) {
                            Some(d) if d.visibility == Visibility::Exported => {
                                if bindings.contains_key(&name.lexeme) {
                                    self.error(
                                        name,
                                        format!("Conflicting imported name '{}'.", name.lexeme)
                                    );
                                } else {
                                    bindings.insert(name.lexeme.clone(), d.symbol.clone());
                                }
                            }
                            Some(_) =>
                                self.error(
                                    name,
                                    format!(
                                        "Module declaration '{}' is private and cannot be imported.",
                                        name.lexeme
                                    )
                                ),
                            None =>
                                self.error(
                                    name,
                                    format!("Module does not declare '{}'.", name.lexeme)
                                ),
                        }
                    }
                }
            }
        }
        let mut resolver = Names::new(&bindings, &self.prelude, id);
        // Only this module's enum cases introduce unqualified constructors.
        for stmt in &stmts {
            let stmt = if let Stmt::Export { declaration, .. } = stmt {
                declaration.as_ref()
            } else {
                stmt
            };
            if let Stmt::Enum { name, cases, .. } = stmt {
                for case in cases {
                    let ctor = (bindings[&name.lexeme].clone(), case.name.lexeme.clone());
                    resolver.constructors
                        .entry(case.name.lexeme.clone())
                        .and_modify(|v| {
                            *v = None;
                        })
                        .or_insert(Some(ctor));
                }
            }
        }
        let mut result = vec![];
        for stmt in stmts {
            if let Stmt::Using { keyword, .. } = stmt {
                result.extend(pending.remove(&keyword.id).unwrap_or_default());
                continue;
            }
            let mut stmt = if let Stmt::Export { declaration, .. } = stmt {
                *declaration
            } else {
                stmt
            };
            resolver.statements(std::slice::from_mut(&mut stmt));
            result.push(stmt);
        }
        self.loading.remove(path);
        result
    }
    pub fn load_module(&mut self, path: PathBuf) -> Vec<Stmt> {
        if self.loading.contains(&path) {
            eprintln!(
                "Warning: Circular import detected for '{}'. Skipping to avoid infinite loop.",
                path.display()
            );
            return vec![];
        }
        if !self.emitted.insert(path.clone()) {
            return vec![];
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                self.error(
                    &Token::synthetic(TokenType::Using, "using"),
                    format!("Error reading '{}': {e}", path.display())
                );
                return vec![];
            }
        };
        let filename = path.to_string_lossy().into_owned();
        self.sources.insert(filename.clone(), source.clone());
        let mut scanner = Scanner::new(&source, &filename);
        scanner.scan_tokens();
        for e in scanner.errors {
            self.parse_errors.push(ParseError {
                token: Token::new(
                    TokenType::Eof,
                    "".into(),
                    Literal::None,
                    e.line,
                    e.column,
                    std::sync::Arc::new(filename.clone())
                ),
                message: e.message,
                hints: vec![],
            });
        }
        let mut parser = Parser::new(scanner.tokens);
        let stmts = parser.parse();
        self.parse_errors.extend(parser.errors);
        let id = self.next_module;
        self.next_module += 1;
        self.resolve_unit(stmts, &path, path.parent().unwrap_or(Path::new(".")), id)
    }
}
fn decl_name(s: &Stmt) -> Option<&Token> {
    match s {
        Stmt::Export { declaration, .. } => decl_name(declaration),
        | Stmt::Function { name, .. }
        | Stmt::ExternFunction { name, .. }
        | Stmt::Struct { name, .. }
        | Stmt::Enum { name, .. }
        | Stmt::Trait { name, .. }
        | Stmt::Alias { name, .. }
        | Stmt::Var { name, .. } => Some(name),
        _ => None,
    }
}
fn stmt_token(s: &Stmt) -> Option<&Token> {
    decl_name(s).or(match s {
        | Stmt::Using { keyword, .. }
        | Stmt::For { keyword, .. }
        | Stmt::ForIn { keyword, .. }
        | Stmt::Return { keyword, .. }
        | Stmt::Throw { keyword, .. }
        | Stmt::Impl { keyword, .. } => Some(keyword),
        _ => None,
    })
}
