#![allow(dead_code)]
mod token;
mod scanner;
mod ast;
mod parser;
mod type_checker;
mod ir;
mod ir_builder;
mod ir_optimizer;
mod backend;
mod module_loader;
mod diagnostics;

use clap::Parser as ClapParser;

use scanner::Scanner;
use parser::Parser;
use type_checker::TypeChecker;
use ir_builder::IrBuilder;
use ir_optimizer::IrOptimizer;
use backend::LlvmEmitter;
use module_loader::ModuleLoader;
use diagnostics::{ DiagnosticRenderer, make_error };
use inkwell::context::Context;

const STDLIB_CODE: &str = include_str!("stdlib.fsh");

#[derive(ClapParser, Debug)]
#[command(
    author,
    version = "0.1.0",
    about = "Fishy's compiler",
    long_about = None
)]
struct Cli {
    /// The .fishy file to compile
    input: String,

    /// Name of the executable
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

fn main() {
    let cli = Cli::parse();

    let file_path = std::path::Path::new(&cli.input);
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

    let mut renderer = DiagnosticRenderer::new();
    renderer.add_file("stdlib.fsh".to_string(), STDLIB_CODE);
    renderer.add_file(cli.input.clone(), &source_code);

    // --- FRONTEND ---

    let mut std_scanner = Scanner::new(STDLIB_CODE, "stdlib.fsh");
    std_scanner.scan_tokens();
    let mut std_parser = Parser::new(std_scanner.tokens);
    let mut full_ast = std_parser.parse();

    if !std_parser.errors.is_empty() {
        println!("Internal error in FishyC's standard library!");
        for err in &std_parser.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }
        std::process::exit(1);
    }

    let mut scanner = Scanner::new(&source_code, &cli.input);
    scanner.scan_tokens();

    let mut parser = Parser::new(scanner.tokens);
    let raw_ast = parser.parse();

    if !parser.errors.is_empty() {
        for err in &parser.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }
        std::process::exit(1);
    }

    full_ast.extend(raw_ast.clone());

    let mut loader = ModuleLoader::new();
    let mut ast = loader.resolve(full_ast, &base_dir);

    let mut checker = TypeChecker::new();
    checker.check(&ast);

    if !checker.errors.is_empty() {
        for err in &checker.errors {
            let diag = make_error(&err.token, &err.message, err.hints.clone());
            renderer.emit(&diag);
        }
        std::process::exit(1);
    }

    ast.extend(checker.instantiations.clone());

    if cli.dump_ast {
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

    // --- MIDDLEWARE (IR) ---

    let builder = IrBuilder::new(
        checker.property_indices,
        checker.resolved_constructors,
        checker.traits,
        checker.resolved_methods,
        checker.trait_vtable_layout,
        checker.struct_sizes,
    );
    let mut ir_module = builder.build(&ast);

    if cli.optimize {
        IrOptimizer::optimize(&mut ir_module);
    }

    if cli.dump_ir {
        let ir_filename = format!("{}.ir", file_stem);
        let mut ir_content = String::new();
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

    if let Err(err_msg) = emitter.compile(&ir_module) {
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
            vec!["LLVM module verification rejected the generated code.".to_string()]
        );
        renderer.emit(&diag);
        std::process::exit(1);
    }
}
