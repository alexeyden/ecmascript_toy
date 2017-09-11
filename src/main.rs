use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

extern crate byteorder;
extern crate getopts;

use getopts::{Options, Matches};

mod tokenizer;
mod parser;
mod syntax_tree;
mod frame_stack;
mod var_analyzer;
mod assembler;
mod util;
mod compiler;

use tokenizer::Tokenizer;
use parser::Parser;
use compiler::Compiler;
use util::GraphvizVisitor;

fn process(matches: &Matches) {
  let source_path = matches.free[0].to_string();

  let mut text = String::new();
  File::open(Path::new(&source_path))
    .unwrap()
    .read_to_string(&mut text).unwrap();
  
  let mut tokenizer = Tokenizer::new(&text);

  let tokens = match &tokenizer.tokenize() {
    &Ok(tokens) => tokens,
    &Err(ref msg) => {
      println!("Tokenizer error:\n{}", msg);
      return;
    }
  };

  if matches.opt_present("t") {
    let mut write : Box<std::io::Write> = if let Some(path) = matches.opt_str("o") {
      Box::new(File::create(Path::new(&path)).unwrap())
    } else {
      Box::new(std::io::stderr())
    };
      
    let mut i = 1;
    for ref t in tokens.iter() {
      writeln!(write, "#{:<4 } {:<30 } at {:>3 },{:>3} {:?}", i, t.text, t.line, t.col, t.type_).unwrap();
      i += 1;
    }

    return;
  }

  let mut parser = Parser::new(tokens);
  let mut ast = parser.parse();

  if matches.opt_present("p") {
    let mut graphviz = GraphvizVisitor::new();
    
    graphviz.begin();
    ast.visit(&mut graphviz);
    graphviz.end();

    let text = format!("// Source: {}\n{}", source_path, graphviz.text());

    if let Some(path) = matches.opt_str("o") {
      File::create(Path::new(&path)).unwrap().write_all(text.as_bytes()).unwrap()
    } else {
      println!("{}", text);
    }

    return;
  }

  let bin_path = if let Some(path) = matches.opt_str("o") {
    path
  } else {
    let stem = Path::new(&source_path).file_stem().unwrap();
    stem.to_str().unwrap().to_string() + ".bin"
  };

  let asm_file = if let Some(asm_path) = matches.opt_str("s") {
    Some(File::create(Path::new(&asm_path)).unwrap())
  } else {
    None
  };
  
  let mut f = File::create(bin_path).unwrap();
  let mut compiler = Compiler::new(&mut f, asm_file);
  compiler.compile(&mut ast);
}

fn main() {
  let args: Vec<String> = env::args().collect();

  let mut opts = Options::new();
  opts.optflag("c", "compile", "compile source file");
  opts.optflag("p", "parse", "parse source file to AST");
  opts.optflag("t", "tokenize", "tokenize source file");
  opts.optflag("h", "help", "show usage");
  opts.optopt("o", "output", "output file", "OUT_FILE");
  opts.optopt("s", "assembly", "assembly output file", "ASM_OUT_FILE");

  let brief = format!("Usage: {} FILE [options]", &args[0]);

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => {
      print!("{}", opts.usage(&brief));
      println!("\nWrong arguments: {}", f.to_string());
      return;
    }
  };

  if matches.opt_present("h") {
    print!("{}", opts.usage(&brief));
    return;
  }

  if matches.free.len() == 0 {
      print!("{}", opts.usage(&brief));
      println!("\nWrong arguments: source file not specified");
      return;
  }

  process(&matches);
}

