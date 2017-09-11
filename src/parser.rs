use std::collections::LinkedList;

use tokenizer::Token;
use tokenizer::TokenType;
use syntax_tree::Node;
use syntax_tree::NodeType;
use syntax_tree::OpType;

pub struct Parser<'a> {
  stream: LinkedList<Token<'a>>,
  token: Token<'a>,
  prev_token: Token<'a>
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &LinkedList<Token<'a>>) -> Parser<'a> {
    Parser {
      stream: tokens.clone(),
      token: Token::new_empty(),
      prev_token: Token::new_empty()
    }
  }

  pub fn parse(&mut self) -> Node {
    self.parse_program()
  }

  fn parse_fun(&mut self, parent: &mut Node) {
    let mut node = self.node_create(NodeType::Function);
    let mut args = self.node_create(NodeType::Block);
    let mut body = self.node_create(NodeType::Block);
    
    self.token_next();
    
    self.token_expect(&TokenType::LPar);
    
    if self.token.type_ != TokenType::RPar {
      loop {
        if self.token.type_ == TokenType::Sym {
          args.body.push(self.node_create(NodeType::Symbol(self.token.text.to_string())));
          self.token_next();
        } else {
          self.die("function argument", &self.token);
        };
        
        if !self.token_accept(&TokenType::Comma) { break; }
      } 
    }
    
    self.token_expect(&TokenType::RPar);
    self.parse_block(&mut body);
    
    node.body.push(args);
    node.body.push(body);
    parent.body.push(node);
  }

  fn parse_factor(&mut self, parent: &mut Node) {
    if self.token.type_ == TokenType::Sym {
      let s = self.token.text;
      self.token_next();

      if s == "fn" || s == "function" {
        self.token_revert();
        self.parse_fun(parent);
      }
      else {
        let sym = self.node_create(NodeType::Symbol(s.to_string()));
        parent.body.push(sym);
      }
    }
    else if self.token.type_ == TokenType::Num {
      let x = self.token.text;
      self.token_next();

      let node = self.node_create(NodeType::Number(x.parse::<f32>().unwrap()));
      parent.body.push(node);
    }
    else if self.token.type_ == TokenType::Str {
      let x = self.token.text;
      self.token_next();

      let string = x.trim_matches('\'').to_string();
      let node = self.node_create(NodeType::String(string));
      parent.body.push(node);
    }
    else if self.token.type_ == TokenType::LPar {
      self.token_next();
      self.parse_condition(parent);
      self.token_expect(&TokenType::RPar);
    }
    else if self.token.type_ == TokenType::LBr {
      self.token_next();
      let mut node = self.node_create(NodeType::Array);
      if self.token.type_ != TokenType::RBr {
        self.parse_list(&mut node);
      }
      parent.body.push(node);
      self.token_expect(&TokenType::RBr);
    }
    else if self.token.type_ == TokenType::LBlock {
      self.token_next();
      let mut node = self.node_create(NodeType::Dict);
      if self.token.type_ != TokenType::RBlock {
        self.parse_dict(&mut node);
      }
      parent.body.push(node);
      self.token_expect(&TokenType::RBlock);
    }
    else {
      self.die("function call or expression", &self.token);
    }
  }

  fn parse_unary(&mut self, parent: &mut Node) {
    let node = match self.token.type_ {
      TokenType::OpPlus  => Some(self.node_create(NodeType::Op(OpType::OpPlus))),
      TokenType::OpMinus => Some(self.node_create(NodeType::Op(OpType::OpMinus))),
      TokenType::OpNot   => Some(self.node_create(NodeType::Op(OpType::OpNot))),
      _ => None
    };

    if let Some(mut n) = node {
      self.token_next();
      self.parse_unary(&mut n);
      parent.body.push(n);
    } else {
      self.parse_call(parent);
    }
  }

  fn parse_list(&mut self, parent: &mut Node) {
    self.parse_condition(parent);

    while self.token_accept(&TokenType::Comma) {
      self.parse_condition(parent);
    }
  }

  fn parse_pair(&mut self, parent: &mut Node) {
    if self.token.type_ == TokenType::Num  {
      parent.body.push(self.node_create(NodeType::Number(self.token.text.parse::<f32>().unwrap())));
    } else if self.token.type_ == TokenType::Sym {
      parent.body.push(self.node_create(NodeType::Symbol(self.token.text.to_string())));
    } else if self.token.type_ == TokenType::Str {
      let string = self.token.text.trim_matches('\'').to_string();
      parent.body.push(self.node_create(NodeType::String(string)));
    } else {
      self.die("symbol or number", &self.token);
    }

    self.token_next();
    self.token_expect(&TokenType::Colon);

    self.parse_condition(parent);
  }
  
  fn parse_dict(&mut self, parent: &mut Node) {
    self.parse_pair(parent);

    while self.token_accept(&TokenType::Comma) {
      self.parse_pair(parent);
    }
  }

  fn parse_accessor(&mut self, parent: &mut Node) {
    let mut node = self.node_create(NodeType::Empty);
    self.parse_factor(&mut node);

    loop {
      if self.token_accept(&TokenType::LBr) {
        let mut member = self.node_create(NodeType::Index);

        self.parse_condition(&mut member);

        if node.type_ == NodeType::Empty {
          member.body.append(&mut node.body);
        } else {
          member.body.push(node);
        }

        self.token_expect(&TokenType::RBr);
        node = member;
      } else if self.token_accept(&TokenType::Dot) {
        if self.token.type_ == TokenType::Sym {
          let mut member = self.node_create(NodeType::Member);
          let sym_node = self.node_create(NodeType::Symbol(self.token.text.to_string()));
          member.body.push(sym_node);

          if node.type_ == NodeType::Empty {
            member.body.append(&mut node.body);
          } else {
            member.body.push(node);
          }

          node = member;
          self.token_next();
        } else {
          self.die("symbol", &self.token);
        }
      } else {
        break;
      }
    }

    if node.type_ == NodeType::Empty {
      parent.body.append(&mut node.body);
    } else {
      parent.body.push(node);
    }
  }
  
  fn parse_call(&mut self, parent: &mut Node) {
    let mut node = self.node_create(NodeType::Empty);
    self.parse_accessor(&mut node);

    loop {
      if self.token_accept(&TokenType::LPar) {
        let mut call = self.node_create(NodeType::Call);
        if node.type_ == NodeType::Empty {
          call.body.append(&mut node.body);
        } else {
          call.body.push(node);
        }

        let mut args = self.node_create(NodeType::Block);
        if self.token.type_ != TokenType::RPar {
          self.parse_list(&mut args);
        }
        call.body.push(args);

        node = call;
        self.token_expect(&TokenType::RPar);
      } else if self.token_accept(&TokenType::Dot) {
        if self.token.type_ == TokenType::Sym {
          let mut member = self.node_create(NodeType::Member);
          let sym_node = self.node_create(NodeType::Symbol(self.token.text.to_string()));
          member.body.push(sym_node);

          if node.type_ == NodeType::Empty {
            member.body.append(&mut node.body);
          } else {
            member.body.push(node);
          }

          node = member;
          self.token_next();
        } else {
          self.die("symbol", &self.token);
        }
      } else {
        break;
      }
    }

    if node.type_ == NodeType::Empty {
      parent.body.append(&mut node.body);
    } else {
      parent.body.push(node);
    }
  }

  fn parse_term(&mut self, mut parent: &mut Node) {
    loop {
      let mut fac = self.node_create(NodeType::Empty);
      self.parse_unary(&mut fac);
      
      fac.type_ = if self.token.type_ == TokenType::OpMul {
        NodeType::Op(OpType::OpMul)
      } else if self.token.type_ == TokenType::OpDiv {
        NodeType::Op(OpType::OpDiv)
      } else if self.token.type_ == TokenType::OpMod {
        NodeType::Op(OpType::OpMod)
      } else {
        parent.body.append(&mut fac.body);
        break;
      };
      
      parent.body.push(fac);
      let p = parent;
      parent = p.body.last_mut().unwrap();
      
      self.token_next();
    }
  }

  fn parse_expression(&mut self, mut parent: &mut Node) {
    let mut term = self.node_create(NodeType::Empty);
    self.parse_term(&mut term);
    let mut term = term.body.drain(0..).next().unwrap();

    loop {
      let type_ = match self.token.type_ {
        TokenType::OpPlus => NodeType::Op(OpType::OpPlus),
        TokenType::OpMinus => NodeType::Op(OpType::OpMinus),
        _ => {
          parent.body.push(term);
          break;
        }
      };
      let mut new_term = self.node_create(type_);

      self.token_next();

      new_term.body.push(term);
      self.parse_term(&mut new_term);

      term = new_term;
    }
  }

  fn parse_condition_cmp(&mut self, mut parent: &mut Node) {
    let mut expr = self.node_create(NodeType::Empty);
    self.parse_expression(&mut expr);
    let mut expr = expr.body.drain(0..).next().unwrap();

    loop {
      let type_ = match self.token.type_ {
        TokenType::OpLs => NodeType::Op(OpType::OpLs),
        TokenType::OpGt => NodeType::Op(OpType::OpGt),
        TokenType::OpGtEq => NodeType::Op(OpType::OpGtEq),
        TokenType::OpLsEq => NodeType::Op(OpType::OpLsEq),
        TokenType::OpEq => NodeType::Op(OpType::OpEq),
        TokenType::OpNotEq => NodeType::Op(OpType::OpNotEq),
        _ => {
          parent.body.push(expr);
          break;
        }
      };

      self.token_next();

      let mut new_expr = self.node_create(type_);
      new_expr.body.push(expr);
      self.parse_expression(&mut new_expr);

      expr = new_expr;
    }
  }
  
  fn parse_condition_and(&mut self, mut parent: &mut Node) {
    let mut expr = self.node_create(NodeType::Empty);
    self.parse_condition_cmp(&mut expr);
    let mut expr = expr.body.drain(0..).next().unwrap();

    loop {
      let type_ = match self.token.type_ {
        TokenType::OpAnd => NodeType::Op(OpType::OpAnd),
        _ => {
          parent.body.push(expr);
          break;
        }
      };

      self.token_next();

      let mut new_expr = self.node_create(type_);
      new_expr.body.push(expr);
      self.parse_condition_cmp(&mut new_expr);

      expr = new_expr;
    }
  }
  
  fn parse_condition(&mut self, mut parent: &mut Node) {
    let mut expr = self.node_create(NodeType::Empty);
    self.parse_condition_and(&mut expr);
    let mut expr = expr.body.drain(0..).next().unwrap();

    loop {
      let type_ = match self.token.type_ {
        TokenType::OpAnd => NodeType::Op(OpType::OpOr),
        _ => {
          parent.body.push(expr);
          break;
        }
      };

      self.token_next();

      let mut new_expr = self.node_create(type_);
      new_expr.body.push(expr);
      self.parse_condition_and(&mut new_expr);

      expr = new_expr;
    }
  }

  fn parse_assignment(&mut self, parent: &mut Node) {
    let mut node = self.node_create(NodeType::Assign);
    self.parse_condition(&mut node);

    if self.token_accept(&TokenType::Assign) {
      self.parse_condition(&mut node);
      parent.body.push(node);
    } else {
      parent.body.append(&mut node.body);
    }

    self.token_expect(&TokenType::End);
  }

  fn parse_statement(&mut self, parent: &mut Node) {
    let sym = if self.token.type_ == TokenType::Sym {
      self.token.text
    } else {
      self.parse_assignment(parent);
      return;
    };

    if sym == "var" {
      self.token_next();

      let name = if let Some(s) = self.token.as_sym() {
        s.to_string()
      } else { 
        self.die("variable name", &self.token); String::new()
      };

      self.token_next();
      self.token_expect(&TokenType::Assign);

      let mut node = self.node_create(NodeType::StmtVar);

      let sym = self.node_create(NodeType::Symbol(name));
      node.body.push(sym);
      
      self.parse_condition(&mut node);
      self.token_expect(&TokenType::End);
      
      parent.body.push(node);
    }
    else if sym == "if" { 
      let mut node = self.node_create(NodeType::StmtIf);
      let mut if_block = self.node_create(NodeType::Block);

      self.token_next();
      self.token_expect(&TokenType::LPar);
      self.parse_condition(&mut node);
      self.token_expect(&TokenType::RPar);
      self.parse_block(&mut if_block);

      node.body.push(if_block);

      if let Some("else") = self.token.as_sym() {
        node.type_ = NodeType::StmtIfElse;

        let mut else_block = self.node_create(NodeType::Block);
        self.token_next();
        self.parse_block(&mut else_block);

        node.body.push(else_block);
      }

      parent.body.push(node);
    }
    else if sym == "while" { 
      let mut node = self.node_create(NodeType::StmtWhile);
      let mut block = self.node_create(NodeType::Block);
      
      self.token_next();
      self.token_expect(&TokenType::LPar);
      self.parse_condition(&mut node);
      self.token_expect(&TokenType::RPar);
      self.parse_block(&mut block);

      node.body.push(block);
      parent.body.push(node);
    }
    else if sym == "return" {
      self.token_next();

      let mut node = self.node_create(NodeType::StmtReturn);

      self.parse_condition(&mut node);

      parent.body.push(node);

      self.token_expect(&TokenType::End);
    }
    else {
      self.parse_assignment(parent);
    }
  }

  fn parse_block(&mut self, parent: &mut Node) {
    if self.token_accept(&TokenType::LBlock) {
      while self.token.type_ != TokenType::RBlock {
        self.parse_block(parent);
      }
      self.token_expect(&TokenType::RBlock);
    }
    else {
      self.parse_statement(parent);
    }
  }

  fn parse_program(&mut self) -> Node {
    self.token_next();

    let mut root = self.node_create(NodeType::Block); 

    while self.token.type_ != TokenType::Eof {
      self.parse_block(&mut root);
    }

    self.token_expect(&TokenType::Eof);

    root
  }

  fn token_next(&mut self) {
    self.prev_token = self.token.clone();
    if let Some(t) = self.stream.pop_front() {
      self.token = t;
    };
  }

  fn token_revert(&mut self) {
    self.stream.push_front(self.token.clone());
    self.token = self.prev_token.clone();
  }

  fn token_accept(&mut self, token: &TokenType) -> bool {
    let accepted = self.token.type_ == *token;

    if accepted {
      self.token_next();
    }

    accepted
  }

  fn token_expect(&mut self, token: &TokenType) {
    if !self.token_accept(token) {
      self.die(&format!("token type '{:?}'", token), &self.token);
    }
  }

  fn die(&self, expected: &str, token: &Token) {
    panic!(format!("Unexpected token '{}' at {},{} (expected {})",
                   token.text, token.line, token.col, expected));
  }

  fn node_create(&mut self, type_: NodeType) -> Node {
    Node::new(type_)
  }
}

