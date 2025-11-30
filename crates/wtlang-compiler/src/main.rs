mod codegen;

use wtlang_core::{Lexer, Parser, SemanticAnalyzer};
use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(ClapParser)]
#[command(name = "wtc")]
#[command(about = "WTLang Compiler - Compile WTLang to Streamlit applications", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build WTLang source files to Python/Streamlit
    Build {
        /// Input WTLang source file
        input: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },
    
    /// Check WTLang source for errors without generating code
    Check {
        /// Input WTLang source file
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build { input, output } => {
            build_command(input, output)?;
        },
        Commands::Check { input } => {
            check_command(input)?;
        },
    }
    
    Ok(())
}

fn build_command(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("Compiling {} to {}", input.display(), output.display());
    
    // Read source file
    let source = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;
    
    // Lexical analysis
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()
        .map_err(|diag| {
            eprintln!("\nLexical errors found:\n{}", diag.format_all());
            anyhow::anyhow!("Lexical analysis failed")
        })?;
    
    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()
        .map_err(|diag| {
            eprintln!("\nSyntax errors found:\n{}", diag.format_all());
            anyhow::anyhow!("Parsing failed")
        })?;
    
    println!("Successfully parsed {} items", program.items.len());
    
    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&program) {
        eprintln!("\nSemantic errors found:");
        for error in &errors {
            eprintln!("  - {}", error);
        }
        return Err(anyhow::anyhow!("Semantic analysis failed with {} error(s)", errors.len()));
    }
    
    println!("✓ Semantic analysis passed");
    
    // Code generation
    let mut codegen = codegen::CodeGenerator::new();
    let output_files = codegen.generate(&program)
        .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;
    
    // Create output directory
    fs::create_dir_all(&output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;
    
    // Write output files
    for (filename, code) in output_files {
        let output_path = output.join(&filename);
        fs::write(&output_path, code)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
        println!("Generated: {}", output_path.display());
    }
    
    // Generate requirements.txt
    let requirements = "streamlit>=1.28.0\npandas>=2.0.0\nopenpyxl>=3.1.0\n";
    let req_path = output.join("requirements.txt");
    fs::write(&req_path, requirements)
        .with_context(|| format!("Failed to write requirements.txt: {}", req_path.display()))?;
    println!("Generated: {}", req_path.display());
    
    println!("\n✓ Compilation successful!");
    println!("\nTo run your application:");
    println!("  cd {}", output.display());
    println!("  pip install -r requirements.txt");
    println!("  streamlit run <PageName>.py");
    
    Ok(())
}

fn check_command(input: PathBuf) -> Result<()> {
    println!("Checking {} for errors", input.display());
    
    // Read source file
    let source = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;
    
    // Lexical analysis
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()
        .map_err(|diag| {
            eprintln!("\nLexical errors found:\n{}", diag.format_all());
            anyhow::anyhow!("Lexical analysis failed")
        })?;
    
    println!("✓ Lexical analysis passed ({} tokens)", tokens.len());
    
    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()
        .map_err(|diag| {
            eprintln!("\nSyntax errors found:\n{}", diag.format_all());
            anyhow::anyhow!("Parsing failed")
        })?;
    
    println!("✓ Parsing passed ({} items)", program.items.len());
    
    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&program) {
        eprintln!("\nSemantic errors found:");
        for error in &errors {
            eprintln!("  - {}", error);
        }
        return Err(anyhow::anyhow!("Semantic analysis failed with {} error(s)", errors.len()));
    }
    
    println!("✓ Semantic analysis passed");
    println!("\n✓ No errors found!");
    
    Ok(())
}
