use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

use byteorder::{WriteBytesExt, LittleEndian};

use syntax_tree::NodeType;
use syntax_tree::OpType;

#[derive(Copy, Clone, Debug)]
pub enum OpCode {
  // Stack
  PushNum = 0x20,
  PushStr = 0x21,
  PushInt = 0x22,
  PushFn  = 0x23,
  Take    = 0x24,
  Swap    = 0x25,
  Pop     = 0x26,

  // Memory
  Load = 0x31,
  Store = 0x32,

  // Control
  JumpIf = 0x40,
  Jump = 0x41,
  Call = 0x42,

  // Arithmetic operations
  Add = 0x50,
  Sub = 0x51,
  Mul = 0x52,
  Div = 0x53,
  Mod = 0x54,
  Neg = 0x55,

  // Logic operations
  Lt    = 0x60,
  Gt    = 0x61,
  Eq    = 0x62,
  NotEq = 0x63,
  Leq   = 0x64,
  Geq   = 0x65,
  And   = 0x66,
  Or    = 0x67,
  Not   = 0x68,

  // Dict operations
  Get = 0x70,
  PushDict = 0x71,
  PushArray = 0x72
}

impl OpCode {
  pub fn from_op_node_type(nt: &NodeType) -> Option<OpCode> {
    match nt {
      &NodeType::Op(OpType::OpMul)   => Some(OpCode::Mul),
      &NodeType::Op(OpType::OpDiv)   => Some(OpCode::Div),
      &NodeType::Op(OpType::OpMod)   => Some(OpCode::Mod),
      &NodeType::Op(OpType::OpOr)    => Some(OpCode::Or),
      &NodeType::Op(OpType::OpAnd)   => Some(OpCode::And),
      &NodeType::Op(OpType::OpLs)    => Some(OpCode::Lt),
      &NodeType::Op(OpType::OpGt)    => Some(OpCode::Gt),
      &NodeType::Op(OpType::OpLsEq)  => Some(OpCode::Leq),
      &NodeType::Op(OpType::OpGtEq)  => Some(OpCode::Geq),
      &NodeType::Op(OpType::OpEq)    => Some(OpCode::Eq),
      &NodeType::Op(OpType::OpNotEq) => Some(OpCode::NotEq),
      &NodeType::Op(OpType::OpNot)   => Some(OpCode::Not),
      &NodeType::Op(OpType::OpPlus)  => Some(OpCode::Add),
      &NodeType::Op(OpType::OpMinus) => Some(OpCode::Sub),
      _ => None
    }
  }
}

pub struct Assembler<'a> {
  file: &'a mut File,
  asm_file: Option<File>,
  sp: Vec<i32>,
  labels: Vec<Vec<u32>>
}

impl<'a> Assembler<'a> {
  pub fn new(f: &'a mut File, asm_f: Option<File>) -> Assembler<'a> {
    Assembler {
      file: f,
      asm_file: asm_f,
      sp: vec![0],
      labels: vec![]
    }
  }

  pub fn get_ip(&mut self) -> u32 {
    self.file.seek(SeekFrom::Current(0)).unwrap() as u32
  }
  pub fn get_sp(&self) -> i32 { *self.sp.last().unwrap() }
  pub fn push_sp(&mut self, new: i32) { self.sp.push(new); }
  pub fn pop_sp(&mut self) -> i32 { self.sp.pop().unwrap() }

  fn print_op(&mut self, op_text: String) {
    let ip = self.get_ip();

    if let Some(ref mut file) = self.asm_file {
      writeln!(file, "{:05} {}", ip, op_text).unwrap();
    }
  }
  
  pub fn push_int(&mut self, value: u32) {
    self.print_op(format!("push_int {}", value));

    self.file.write_u8(OpCode::PushInt as u8).unwrap();
    self.file.write_u32::<LittleEndian>(value).unwrap();
    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn push_float(&mut self, value: f32) {
    self.print_op(format!("push_float {}", value));

    self.file.write_u8(OpCode::PushNum as u8).unwrap();
    self.file.write_f32::<LittleEndian>(value).unwrap();
    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn push_str(&mut self, value: &str) {
    self.print_op(format!("push_str \"{}\"", value));

    let length = value.as_bytes().len() as u32;

    self.file.write_u8(OpCode::PushStr as u8).unwrap();
    self.file.write_u32::<LittleEndian>(length).unwrap();
    self.file.write_all(value.as_bytes()).unwrap();

    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn push_fn(&mut self,
                 parent_frames_count: u32,
                 parent_frames_offset: u32,
                 own_frame_size: u32
  ) {
    self.print_op(format!("push_fn {} {} {}",
                          parent_frames_count,
                          parent_frames_offset,
                          own_frame_size));

    self.file.write_u8(OpCode::PushFn as u8).unwrap();
    self.file.write_u32::<LittleEndian>(parent_frames_count).unwrap();
    self.file.write_u32::<LittleEndian>(parent_frames_offset).unwrap();
    self.file.write_u32::<LittleEndian>(own_frame_size).unwrap();
  }

  pub fn push_dict(&mut self, len: u32) {
    self.print_op(format!("push_dict {}", len));

    self.file.write_u8(OpCode::PushDict as u8).unwrap();
    self.file.write_u32::<LittleEndian>(len).unwrap();

    *self.sp.last_mut().unwrap() -= len as i32 * 2;
    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn push_array(&mut self, len: u32) {
    self.print_op(format!("push_array {}", len));

    self.file.write_u8(OpCode::PushArray as u8).unwrap();
    self.file.write_u32::<LittleEndian>(len).unwrap();

    *self.sp.last_mut().unwrap() -= len as i32;
    *self.sp.last_mut().unwrap() += 1;
  }
    
  pub fn take(&mut self, offset: u32) {
    self.print_op(format!("take {}", offset));

    self.file.write_u8(OpCode::Take as u8).unwrap();
    self.file.write_u32::<LittleEndian>(offset).unwrap();

    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn swap(&mut self, a: u32, b: u32) {
    self.print_op(format!("swap {} {}", a, b));

    self.file.write_u8(OpCode::Swap as u8).unwrap();
    self.file.write_u32::<LittleEndian>(a).unwrap();
    self.file.write_u32::<LittleEndian>(b).unwrap();
  }

  pub fn pop(&mut self, n: u32) {
    self.print_op(format!("pop {}", n));

    self.file.write_u8(OpCode::Pop as u8).unwrap();
    self.file.write_u32::<LittleEndian>(n).unwrap();

    *self.sp.last_mut().unwrap() -= n as i32;
  }

  pub fn load(&mut self, offset: u32) {
    self.print_op(format!("load {}", offset));

    self.file.write_u8(OpCode::Load as u8).unwrap();
    self.file.write_u32::<LittleEndian>(offset).unwrap();
  }
  
  pub fn store(&mut self) {
    self.print_op("store".to_string());

    self.file.write_u8(OpCode::Store as u8).unwrap();

    *self.sp.last_mut().unwrap() -= 2;
  }

  pub fn op_binary(&mut self, op: &NodeType) {
    self.print_op(format!("op {:?}", op));

    let opcode = OpCode::from_op_node_type(op).unwrap();
    self.file.write_u8(opcode as u8).unwrap();

    *self.sp.last_mut().unwrap() -= 1;
  }

  pub fn op_unary(&mut self, op: &NodeType) {
    self.print_op(format!("op {:?}", op));

    let op = match op {
      &NodeType::Op(OpType::OpPlus) => return,
      &NodeType::Op(OpType::OpMinus) => OpCode::Neg,
      &NodeType::Op(OpType::OpNot) => OpCode::Not,
      _ => panic!()
    };
    self.file.write_u8(op as u8).unwrap();
  }

  pub fn gen_label(&mut self) -> usize {
    self.labels.push(vec![]);
    self.labels.len() - 1
  }

  pub fn put_label(&mut self, label: usize) {
    self.print_op(format!("push_int @label_{}", label));

    let ip = self.get_ip();
    self.labels[label].push(ip);

    self.file.write_u8(OpCode::PushInt as u8).unwrap();
    self.file.write_u32::<LittleEndian>(0xDEAD).unwrap();
    *self.sp.last_mut().unwrap() += 1;
  }

  pub fn fill_label(&mut self, label: usize) {
    self.print_op(format!("@label_{}:", label));

    let offset = self.get_ip(); 
    for pos in self.labels[label].iter() {
      self.file.seek(SeekFrom::Start(*pos as u64)).unwrap();
      self.file.write_u8(OpCode::PushInt as u8).unwrap();
      self.file.write_u32::<LittleEndian>(offset as u32).unwrap();
      self.file.seek(SeekFrom::End(0)).unwrap();
    }
  }

  pub fn jump(&mut self) {
    self.print_op("jump".to_string());

    self.file.write_u8(OpCode::Jump as u8).unwrap();

    *self.sp.last_mut().unwrap() -= 1;
  }

  pub fn jump_if(&mut self) {
    self.print_op("jump_if".to_string());

    self.file.write_u8(OpCode::JumpIf as u8).unwrap();

    *self.sp.last_mut().unwrap() -= 2;
  }

  pub fn call(&mut self, n_args: u32) {
    self.print_op("call".to_string());

    self.file.write_u8(OpCode::Call as u8).unwrap();
    *self.sp.last_mut().unwrap() -= 1 + n_args as i32 + 1;
  }

  pub fn get(&mut self) {
    self.print_op("get".to_string());

    self.file.write_u8(OpCode::Get as u8).unwrap();
    *self.sp.last_mut().unwrap() -= 1;
  }
}

