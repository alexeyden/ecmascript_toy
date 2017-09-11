use std::collections::HashMap;
use std::fs::File;

use syntax_tree::Node;
use syntax_tree::NodeType;
use syntax_tree::OpType;
use assembler::Assembler;
use frame_stack::FrameStackTree;

use var_analyzer::build_frame_stack;

pub struct Compiler<'a> {
  frame_stack: FrameStackTree,
  assembler: Assembler<'a>,
  sys_objects: HashMap<&'a str, u32>
}

impl<'a> Compiler<'a> {
  pub fn new(file: &'a mut File, asm_file: Option<File>) -> Compiler<'a> {
    Compiler {
      frame_stack: FrameStackTree::new(),
      assembler: Assembler::new(file, asm_file),
      sys_objects: [
        ("std",   0x00),
      ].iter().cloned().collect()
    }
  }

  pub fn compile(&mut self, ast: &mut Node) { 
    self.frame_stack = build_frame_stack(ast);

    let num_global_vars = self.frame_stack.root_frame().var_offsets.len();

    self.assembler.push_int(0);

    let start_label = self.assembler.gen_label();
    self.assembler.put_label(start_label);
    self.assembler.push_fn(0, 0, num_global_vars as u32);

    self.assembler.call(0);

    self.assembler.fill_label(start_label);

    self.compile_block(ast);
  }

  fn compile_block(&mut self, node: &Node) {
    match node.type_ {
      NodeType::Block => {
        for ref stmt in &node.body {
          self.compile_block(&stmt);
        }
      },
      NodeType::Assign |
      NodeType::StmtVar => {
        self.compile_assign(node);
      },
      NodeType::Call => {
        self.compile_call(node);
        self.assembler.pop(1);
      },
      NodeType::StmtIf |
      NodeType::StmtIfElse => {
        self.compile_if(node);
      },
      NodeType::StmtWhile => {
        self.compile_while(node);
      },
      NodeType::StmtReturn => {
        self.compile_return(node);
      },
      _ => {
        panic!("unsupported statement");
      }
    }
  }

  fn compile_assign(&mut self, node: &Node) {
    let lhand_node = node.body.get(0).unwrap();
    let rhand_node = node.body.get(1).unwrap();

    self.compile_expr(rhand_node);
    self.take_value(rhand_node);
    self.compile_expr(lhand_node);
    self.assembler.store();
  }

  fn compile_dict_key(&mut self, node: &Node) {
    match node.type_ {
      NodeType::Symbol(ref name) |
      NodeType::String(ref name) => {
        self.assembler.push_str(name);
      },
      NodeType::Number(num) => {
        self.assembler.push_float(num);
      },
      _ => { panic!("invalid dict key: {:?}", node.type_); }
    }
  }

  fn compile_expr(&mut self, node: &Node) { 
    match &node.type_ {
      &NodeType::Op(OpType::OpMul)     |
      &NodeType::Op(OpType::OpDiv)     |
      &NodeType::Op(OpType::OpMod)     |
      &NodeType::Op(OpType::OpOr)      |
      &NodeType::Op(OpType::OpAnd)     |
      &NodeType::Op(OpType::OpLs)      |
      &NodeType::Op(OpType::OpGt)      |
      &NodeType::Op(OpType::OpLsEq)    |
      &NodeType::Op(OpType::OpGtEq)    |
      &NodeType::Op(OpType::OpEq)      |
      &NodeType::Op(OpType::OpNotEq)   => {
        self.compile_expr(node.body.get(0).unwrap());
        self.take_value(node.body.get(0).unwrap());

        self.compile_expr(node.body.get(1).unwrap());
        self.take_value(node.body.get(1).unwrap());

        self.assembler.op_binary(&node.type_);
      },
      &NodeType::Op(OpType::OpNot)  |
      &NodeType::Op(OpType::OpPlus) => {
        self.compile_expr(node.body.get(0).unwrap());
        self.take_value(node.body.get(0).unwrap());
        
        if let Some(ref right_node) = node.body.get(1) {
          self.compile_expr(right_node);
          self.take_value(right_node);
          self.assembler.op_binary(&node.type_);
        } else {
          self.assembler.op_unary(&node.type_);
        }
      },
      &NodeType::Op(OpType::OpMinus) => {
        if let Some(ref right_node) = node.body.get(1) {
          self.compile_expr(node.body.get(0).unwrap());
          self.take_value(node.body.get(0).unwrap());
          self.compile_expr(right_node);
          self.take_value(right_node);
          self.assembler.op_binary(&node.type_);
        } else {
          if let NodeType::Number(n) = node.body.get(0).unwrap().type_ {
            self.assembler.push_float(-n);
          } else {
            self.compile_expr(node.body.get(0).unwrap());
            self.take_value(node.body.get(0).unwrap());
            self.assembler.op_unary(&node.type_);
          }
        }
      },
      &NodeType::Member => {
        self.compile_expr(node.body.get(1).unwrap());
        self.take_value(node.body.get(1).unwrap());

        self.compile_dict_key(node.body.get(0).unwrap());

        self.assembler.get();
      },
      &NodeType::Index => {
        self.compile_expr(node.body.get(1).unwrap());
        self.take_value(node.body.get(1).unwrap());

        self.compile_expr(node.body.get(0).unwrap());
        self.take_value(node.body.get(0).unwrap());

        self.assembler.get();
      },
      &NodeType::Dict => {
        for kv in node.body.chunks(2) {
          let (k, val) = (&kv[0], &kv[1]);
          self.compile_dict_key(k);
          self.compile_expr(val);
          self.take_value(val);
        }
        self.assembler.push_dict(node.body.len() as u32 / 2);
      },
      &NodeType::Array => {
        for val in node.body.iter() {
          self.compile_expr(val);
          self.take_value(val);
        }
        self.assembler.push_array(node.body.len() as u32);
      },
      &NodeType::Number(n) => {
        self.assembler.push_float(n);
      },
      &NodeType::String(ref s) => {
        self.assembler.push_str(s);
      },
      &NodeType::Symbol(ref s) => {
        if let Some(&sys_ptr) = self.sys_objects.get::<str>(s) {
          self.assembler.push_int(sys_ptr);
        } else {
          if let Some(var) = self.frame_stack.find_var(s) {
            let sp_offset = self.assembler.get_sp() as u32 - var.frame_offset as u32;

            self.assembler.take(sp_offset);
            self.assembler.push_int(var.var_offset as u32);
            self.assembler.op_binary(&NodeType::Op(OpType::OpPlus));
          } else {
            panic!("No such variable: {}", &s);
          }
        }
      },
      &NodeType::Call => {
        self.compile_call(node);
      },
      &NodeType::Function => {
        self.compile_fn(node);
      },
      _ => panic!()
    }
  }

  fn compile_fn(&mut self, node: &Node) {
    self.frame_stack.enter();
    
    let label_bypass = self.assembler.gen_label();
    let label_begin = self.assembler.gen_label();

    // push fn address & parent frames 

    let parents_len = self.frame_stack.parents().len() as u32;

    let frame_size = {
      let frame = self.frame_stack.cur_frame();
      let frame = &self.frame_stack.frames()[frame];
      frame.var_offsets.len() as u32
    };

    let sp = self.assembler.get_sp() as u32 + 1;
    
    self.assembler.put_label(label_begin);
    self.assembler.push_fn(parents_len, sp, frame_size);

    // setup bypass jump
    
    self.assembler.put_label(label_bypass);
    self.assembler.jump();

    self.assembler.fill_label(label_begin);

    // function body 

    self.assembler.push_sp(parents_len as i32);

    let body = node.body.get(1).unwrap();
    self.compile_block(body);

    // clean up stack and jump back

    let sp = self.assembler.get_sp();
    self.assembler.pop(sp as u32 + 1);
    self.assembler.pop_sp();

    self.assembler.push_int(0);
    self.assembler.swap(0, 1);
    self.assembler.jump();

    self.assembler.fill_label(label_bypass);

    self.frame_stack.exit();
  }

  fn compile_return(&mut self, node: &Node) {
    let sp = self.assembler.get_sp();

    self.assembler.push_sp(sp);

    if node.body.len() > 0 {
      self.compile_expr(&node.body[0]);
      self.take_value(&node.body[0]);
    } else {
      self.assembler.push_int(0);
    }
    
    self.assembler.swap(0, sp as u32 + 1);
    self.assembler.pop(sp as u32 + 1);

    self.assembler.swap(0, 1);
    self.assembler.jump();

    self.assembler.pop_sp();
  }

  fn compile_call(&mut self, node: &Node) {
    let ret_label = self.assembler.gen_label();
    self.assembler.put_label(ret_label);

    let addr_node = &node.body[0];
    let args_node = &node.body[1];

    for ref n in &args_node.body {
      self.compile_expr(n);
      self.take_value(n);
    }

    self.assembler.push_int(args_node.body.len() as u32);
    self.compile_expr(&addr_node);

    self.assembler.call(args_node.body.len() as u32);
    self.assembler.fill_label(ret_label);
  }

  fn compile_if(&mut self, node: &Node) {
    let cond = node.body.get(0).unwrap();
    let if_body = node.body.get(1).unwrap();
    
    self.compile_expr(cond);
    self.take_value(cond);
    
    self.assembler.op_unary(&NodeType::Op(OpType::OpNot));

    let else_label = self.assembler.gen_label(); 
    self.assembler.put_label(else_label);
    self.assembler.jump_if();

    self.compile_block(if_body);
    
    let out_label = self.assembler.gen_label();
    self.assembler.put_label(out_label);
    self.assembler.jump();
    
    self.assembler.fill_label(else_label); 
    if let Some(else_body) = node.body.get(2) {
      self.compile_block(else_body);
    }
    self.assembler.fill_label(out_label);
  }
  
  fn compile_while(&mut self, node: &Node) {
    let cond = node.body.get(0).unwrap();
    let body = node.body.get(1).unwrap();

    let begin = self.assembler.get_ip();
    
    self.compile_expr(cond);
    self.take_value(cond);
    self.assembler.op_unary(&NodeType::Op(OpType::OpNot));
    
    let out_label = self.assembler.gen_label();
    self.assembler.put_label(out_label);
    self.assembler.jump_if();

    self.compile_block(body);

    self.assembler.push_int(begin);
    self.assembler.jump();

    self.assembler.fill_label(out_label); 
  }

  fn take_value(&mut self, node: &Node) {
    match node.type_ {
      NodeType::Symbol(_) |
      NodeType::Member |
      NodeType::Index => {
        self.assembler.load(0);
      },
      _ => {}
    }
  }
}

