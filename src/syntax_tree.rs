use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub enum OpType {
  OpPlus,
  OpMinus,
  OpMul,
  OpDiv,
  OpMod,
  OpOr,
  OpAnd,
  OpNot,
  OpLs,
  OpGt,
  OpLsEq,
  OpGtEq,
  OpEq,
  OpNotEq
}

impl fmt::Debug for OpType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let names = [ "+", "-", "*", "/", "%", "||", "&&", "!", "<", ">", "<=", ">=", "==", "!=" ];
    write!(f, "{}", names[*self as usize])
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
  Number(f32),
  String(String),
  Symbol(String),
  Function,
  Call,
  Dict,
  Array,
  StmtVar, StmtIf, StmtIfElse, StmtWhile, StmtReturn,
  Member,
  Index,
  Op(OpType),
  Assign,
  Block,
  Empty
}

#[derive(Clone, Debug)]
pub struct Node {
  pub type_: NodeType,
  pub body: Vec<Node>,
}

#[allow(unused_variables)]
pub trait Visitor {
  fn enter_term(&mut self, node: &mut Node) {}
  fn enter_fun(&mut self, node: &mut Node) {}
  fn enter_call(&mut self, node: &mut Node) {}
  fn enter_var(&mut self, node: &mut Node) {}
  fn enter_if(&mut self, node: &mut Node) {}
  fn enter_while(&mut self, node: &mut Node) {}
  fn enter_return(&mut self, node: &mut Node) {}
  fn enter_expr(&mut self, node: &mut Node) {}
  fn enter_assign(&mut self, node: &mut Node) {}
  fn enter_block(&mut self, node: &mut Node) {}

  fn exit_term(&mut self, node: &mut Node) {}
  fn exit_fun(&mut self, node: &mut Node) {}
  fn exit_call(&mut self, node: &mut Node) {}
  fn exit_var(&mut self, node: &mut Node) {}
  fn exit_if(&mut self, node: &mut Node) {}
  fn exit_while(&mut self, node: &mut Node) {}
  fn exit_return(&mut self, node: &mut Node) {}
  fn exit_expr(&mut self, node: &mut Node) {}
  fn exit_assign(&mut self, node: &mut Node) {}
  fn exit_block(&mut self, node: &mut Node) {}

  fn visit(&mut self, node: &mut Node) {}
}

impl Node {
  pub fn new(type_: NodeType) -> Node {
    Node { type_: type_, body: vec![] }
  }

  pub fn visit(&mut self, visitor: &mut Visitor) {
    match self.type_ {
      NodeType::Number(_) |
      NodeType::String(_) |
      NodeType::Symbol(_) =>
        visitor.enter_term(self),
      NodeType::Function =>
        visitor.enter_fun(self),
      NodeType::Call =>
        visitor.enter_call(self),
      NodeType::StmtVar =>
        visitor.enter_var(self),
      NodeType::StmtIf | NodeType::StmtIfElse =>
        visitor.enter_if(self),
      NodeType::StmtWhile =>
        visitor.enter_while(self),
      NodeType::StmtReturn =>
        visitor.enter_return(self),
      NodeType::Op(_) => 
        visitor.enter_expr(self),
      NodeType::Assign =>
        visitor.enter_assign(self),
      NodeType::Block =>
        visitor.enter_block(self),
      _ => {}
    }

    visitor.visit(self);

    for ref mut ch in self.body.iter_mut() {
      ch.visit(visitor);
    }

    match self.type_ {
      NodeType::Number(_) |
      NodeType::String(_) |
      NodeType::Symbol(_) =>
        visitor.exit_term(self),
      NodeType::Function =>
        visitor.exit_fun(self),
      NodeType::Call =>
        visitor.exit_call(self),
      NodeType::StmtVar =>
        visitor.exit_var(self),
      NodeType::StmtIf | NodeType::StmtIfElse =>
        visitor.exit_if(self),
      NodeType::StmtWhile =>
        visitor.exit_while(self),
      NodeType::StmtReturn =>
        visitor.exit_return(self),
      NodeType::Op(_) => 
        visitor.exit_expr(self),
      NodeType::Assign =>
        visitor.exit_assign(self),
      NodeType::Block =>
        visitor.exit_block(self),
      _ => {}
    }
  }
}

