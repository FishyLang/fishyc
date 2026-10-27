#![allow(dead_code)]

mod ast;
mod captures;
mod identity;
mod type_properties;
mod ownership_ir;
mod ownership;

mod ast_visit;
mod backend;
mod diagnostics;
mod ir;
mod ir_builder;
mod ir_optimizer;
mod ir_verify;
mod module_loader;
mod module_names;
mod parser;
mod scanner;
mod symbols;
mod token;
mod type_checker;

use clap::Parser as ClapParser;

use backend::LlvmEmitter;
use diagnostics::{ make_error, DiagnosticRenderer };
use inkwell::context::Context;
use ir_builder::IrBuilder;
use ir_optimizer::IrOptimizer;
use module_loader::ModuleLoader;
use parser::Parser;
use scanner::Scanner;
use type_checker::TypeChecker;

const STDLIB_CODE: &str = include_str!("stdlib.fih");

#[derive(ClapParser, Debug)]
#[command(author, version = "0.1.0", about = "Fishy's compiler", long_about = None)]
struct Cli {
    /// The .fih file to compile
    input: String,

    /// Path of the object file (link it with a C linker)
    #[arg(short, long, default_value = "fishy_app.o")]
    output: String,

    /// Only checks the code without running it
    #[arg(short, long)]
    check: bool,

    /// Enables Fishy's optimizer
    #[arg(short = 'O', long)]
    optimize: bool,

    /// Saves Fishy's AST into an .ast file
    #[arg(long)]
    dump_ast: bool,

    /// Saves Fishy's IR into an .ir file
    #[arg(long)]
    dump_ir: bool,

    /// Saves the native LLVM IR in an .ll file
    #[arg(long)]
    emit_llvm: bool,
}

fn check_extension(path: &std::path::Path, expected: &str) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case(expected.trim_start_matches('.')))
        .unwrap_or(false)
}

fn main() {
    let cli = Cli::parse();

    let file_path = std::path::Path::new(&cli.input);
    if !check_extension(file_path, ".fih") {
        eprintln!("Error: input file must have '{}' extension.", ".fih");
        std::process::exit(1);
    }

    let source_code = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", cli.input, e);
            std::process::exit(1);
        }
    };

    let base_dir = file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dump");

    let cli_input_path = file_path.to_string_lossy().into_owned();

    let mut renderer = DiagnosticRenderer::new();
    renderer.add_file("stdlib.fih".to_string(), STDLIB_CODE);
    renderer.add_file(cli_input_path.clone(), &source_code);

    // --- FRONTEND ---

    let mut std_scanner = Scanner::new(STDLIB_CODE, "stdlib.fih");
    std_scanner.scan_tokens();
    let mut std_parser = Parser::new(std_scanner.tokens);
    let mut full_ast = std_parser.parse();
    ast_visit::inject_prelude_intrinsics(&mut full_ast);

    if !std_parser.errors.is_empty() {
        eprintln!("Internal error in FishyC's standard library!");

        for err in &std_parser.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }

        std::process::exit(1);
    }

    let mut scanner = Scanner::new(&source_code, &cli.input);
    scanner.scan_tokens();
    if !scanner.errors.is_empty() {
        for error in &scanner.errors {
            eprintln!("{}:{}:{}: {}", cli.input, error.line, error.column, error.message);
        }
        std::process::exit(1);
    }

    let mut parser = Parser::new(scanner.tokens);
    let raw_ast = parser.parse();

    if !parser.errors.is_empty() {
        for err in &parser.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }

        std::process::exit(1);
    }

    let mut loader = ModuleLoader::new();
    let mut ast = loader.resolve_root(raw_ast, &base_dir, full_ast);
    for (path, source) in &loader.sources {
        renderer.add_file(path.clone(), source);
    }

    if !loader.parse_errors.is_empty() {
        for err in &loader.parse_errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }

        std::process::exit(1);
    }

    let mut checker = TypeChecker::new();
    checker.module_symbols = loader.symbols.clone();
    checker.external_names = loader.external_names.clone();
    checker.check(&mut ast);

    if !checker.errors.is_empty() {
        for err in &checker.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }

        std::process::exit(1);
    }

    ast.extend(checker.instantiations.clone());
    let mut evaluators = checker.default_evaluators.iter().collect::<Vec<_>>();
    evaluators.sort_by_key(|(name, _)| *name);
    ast.extend(evaluators.into_iter().map(|(_, stmt)| stmt.clone()));

    let ownership = ownership::Plan::build(
        &ast,
        &(ownership::Context {
            types: &checker.user_types,
            pattern_bindings: &checker.pattern_bindings,
            pattern_views: &checker.pattern_views,
            parametric_bindings: &checker.parametric_bindings,
            calls: &checker.resolved_calls,
            fields: &checker.property_indices,
            captures: &checker.capture_plans,
            defaults: &checker.default_calls,
            specialized: &checker.specialized_methods,
            traits: &checker.traits,
        })
    );
    if !ownership.errors.is_empty() {
        for error in &ownership.errors {
            let hints = error.note
                .as_ref()
                .map(|token|
                    vec![
                        format!(
                            "'{}' is declared at {}:{}",
                            token.lexeme,
                            token.file_path,
                            token.line
                        )
                    ]
                )
                .unwrap_or_default();
            renderer.emit(&make_error(&error.span, &error.message, hints));
        }
        std::process::exit(1);
    }

    if cli.dump_ast && !cli.check {
        let ast_filename = format!("{}.ast", file_stem);
        let ast_content = format!("{:#?}", ast);

        if let Err(e) = std::fs::write(&ast_filename, ast_content) {
            eprintln!("WARNING: Error saving AST to '{}': {}", ast_filename, e);
        } else {
            println!("AST saved to '{}'", ast_filename);
        }
    }

    if cli.check {
        println!("Checking successful. No errors were found.");
        return;
    }

    if cli.dump_ir && !cli.check {
        let mut plans = checker.capture_plans.iter().collect::<Vec<_>>();
        plans.sort_by_key(|(id, _)| **id);
        let content = plans
            .into_iter()
            .map(|(id, bindings)| format!("lambda {id}: {bindings:#?}\n"))
            .collect::<String>();
        if let Err(e) = std::fs::write(format!("{}.captures", file_stem), content) {
            eprintln!("Cannot write capture plans: {e}");
        }
    }
    // --- MIDDLEWARE (IR) ---
    let builder = IrBuilder::new(
        checker.capture_plans,
        checker.property_indices,
        checker.resolved_calls,
        checker.specialized_methods,
        checker.default_calls,
        checker.traits,
        checker.trait_vtable_layout,
        checker.user_types
    ).with_ownership(ownership);

    let mut ir_module = builder.build(&ast);

    if let Err(e) = ir_verify::verify(&ir_module) {
        eprintln!("Internal IR validation error before optimization: {e}");
        std::process::exit(1);
    }
    if cli.optimize {
        IrOptimizer::optimize(&mut ir_module);
        if let Err(e) = ir_verify::verify(&ir_module) {
            eprintln!("Internal IR validation error after optimization: {e}");
            std::process::exit(1);
        }
    }

    if cli.dump_ir {
        let ir_filename = format!("{}.ir", file_stem);
        let mut ir_content = String::new();
        let mut globals: Vec<_> = ir_module.globals.iter().collect();
        globals.sort_by_key(|(r, _)| r.0);
        for (r, t) in globals {
            ir_content.push_str(&format!("global {}: {}\n", r, t));
        }
        let mut layouts: Vec<_> = ir_module.structs.iter().collect();
        layouts.sort_by_key(|(name, _)| *name);
        for (name, fields) in layouts {
            ir_content.push_str(&format!("struct %{}: {:?}\n", name, fields));
        }

        for func in &ir_module.functions {
            ir_content.push_str(&format!("{}\n", func));
        }

        if let Err(e) = std::fs::write(&ir_filename, ir_content) {
            eprintln!("WARNING: Error saving IR to '{}': {}", ir_filename, e);
        } else {
            println!("Fishy IR saved to '{}'", ir_filename);
        }
    }

    // --- BACKEND (LLVM) ---

    let context = Context::create();
    let mut emitter = LlvmEmitter::new(&context, "fishy_module");

    if let Err(err_msg) = emitter.compile(&ir_module, &loader.external_names) {
        let syn_token = token::Token::synthetic(token::TokenType::Eof, "LLVM");

        let diag = make_error(
            &syn_token,
            err_msg,
            vec!["Internal Compiler Error (ICE) during code generation.".to_string()]
        );

        renderer.emit(&diag);
        std::process::exit(1);
    }

    if cli.emit_llvm {
        let llvm_filename = format!("{}.ll", file_stem);
        let llvm_content = emitter.module.print_to_string().to_string();

        if let Err(e) = std::fs::write(&llvm_filename, llvm_content) {
            eprintln!("WARNING: Error saving LLVM IR to '{}': {}", llvm_filename, e);
        } else {
            println!("LLVM IR saved to '{}'", llvm_filename);
        }
    }

    if let Err(err_msg) = emitter.build_executable(&cli.output) {
        let syn_token = token::Token::synthetic(token::TokenType::Eof, "LLVM");

        let diag = make_error(
            &syn_token,
            err_msg,
            vec!["Object emission failed; check the target and output path.".to_string()]
        );

        renderer.emit(&diag);
        std::process::exit(1);
    }
}
