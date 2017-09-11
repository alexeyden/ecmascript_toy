use std::collections::LinkedList;
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
  Sym, Str, Num,
  OpPlus, OpMinus, OpMul, OpDiv, OpMod,
  OpOr, OpAnd, OpNot, OpLs, OpGt, OpLsEq, OpGtEq, OpEq, OpNotEq,
  Assign,
  Comma,
  Dot,
  Colon,
  End,
  LBr, RBr,
  LBlock, RBlock,
  LPar, RPar,
  Comment,
  Empty,
  Eof
}

#[derive(Clone)]
pub struct Token<'a> {
  pub type_: TokenType,
  pub text: &'a str,
  pub line: usize,
  pub col: usize,
}

impl<'a> Token<'a> {
  pub fn new(t: TokenType, text: &'a str, line: usize, col: usize) -> Token<'a> {
    Token {
      type_: t,
      text: text,
      line: line,
      col: col
    }
  }

  pub fn new_empty() -> Token<'a> {
    Token {
      type_: TokenType::Empty,
      text: "", 
      line: 0,
      col: 0
    }
  }

  pub fn as_sym(&self) -> Option<&str> {
    if self.type_ == TokenType::Sym { Some(self.text) } else { None }
  }
}

pub struct Tokenizer<'a> {
  pub tokens: LinkedList<Token<'a>>,
  pub text: &'a str,
  it: Peekable<CharIndices<'a>>,
  line: usize,
  col: usize,
  start: usize,
  token: Token<'a>
}

impl<'a> Tokenizer<'a> {
  pub fn new(text: &'a str) -> Tokenizer<'a> {
    Tokenizer {
      line: 1,
      col: 0,
      tokens: LinkedList::<Token>::new(),
      text: text,
      it: text.char_indices().peekable(),
      start: 0,
      token: Token::new_empty()
    }
  }

  pub fn tokenize(&mut self) -> Result<&LinkedList<Token>, String> {
    loop {
      let c = match self.peek_char() {
        Some(ch) => ch,
        None => break
      };
      
      match self.token.type_ {
        TokenType::Sym => {
          if c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '_' {
            self.next();
          }
          else {
            self.commit();
          }
        },
        TokenType::Num => {
          let cur = self.cur_text();

          let is_valid_num =
            c >= '0' && c <= '9' ||
            c == '.' && !cur.contains(".");

          if is_valid_num {
            self.next();
          } else {
            self.commit();
          }
        },
        TokenType::Str => {
          if c == '\'' {
            self.next();
            self.commit();
          } else {
            self.next();
          }
        },
        TokenType::Comment => {
          if c == '\n' {
            self.next();
            self.reset();
          } else {
            self.next();
          }
        },
        _ => {
          if c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z' {
            self.new_token(TokenType::Sym);
            self.next();
          }
          else if c == '/' {
            self.next();
            if let Some('/') = self.peek_char() {
              self.next();
              self.new_token(TokenType::Comment);
            } else {
              self.new_token(TokenType::OpDiv);
              self.commit();
            }
          }
          else if c == '+' {
            self.new_token(TokenType::OpPlus);
            self.next();
            self.commit();
          }
          else if c == '-' {
            self.new_token(TokenType::OpMinus);
            self.next();
            self.commit();
          }
          else if c >= '0' && c <= '9' {
            self.new_token(TokenType::Num);
            self.next();
          }
          else if c == '\'' {
            self.new_token(TokenType::Str);
            self.next();
          }
          else if c == '=' {
            self.new_token(TokenType::Assign);
            self.next();
              
            if let Some('=') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpEq);
              self.commit();
            }
            else {
              self.commit();
            };
          }
          else if c == '(' {
            self.new_token(TokenType::LPar);

            self.next();
            self.commit();
          }
          else if c == ')' {
            self.new_token(TokenType::RPar);

            self.next();
            self.commit();
          }
          else if c == '[' {
            self.new_token(TokenType::LBr);
            self.next();
            self.commit();
          }
          else if c == ']' {
            self.new_token(TokenType::RBr);
            self.next();
            self.commit();
          }
          else if c == '.' {
            self.new_token(TokenType::Dot);
            self.next();
            self.commit();
          }
          else if c == '{' {
            self.new_token(TokenType::LBlock);
            self.next();
            self.commit();
          }
          else if c == '}' {
            self.new_token(TokenType::RBlock);
            self.next();
            self.commit();
          }
          else if c == ';' {
            self.new_token(TokenType::End);
            self.next();
            self.commit();
          }
          else if c == ':' {
            self.new_token(TokenType::Colon);
            self.next();
            self.commit();
          }
          else if c == ',' {
            self.new_token(TokenType::Comma);
            self.next();
            self.commit();
          }
          else if c == '*' { 
            self.new_token(TokenType::OpMul);
            self.next();
            self.commit();
          }
          else if c == '%' { 
            self.new_token(TokenType::OpMod);
            self.next();
            self.commit();
          }
          else if c == '!' { 
            self.new_token(TokenType::OpNot);
            self.next();
            
            if let Some('=') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpNotEq);
              self.commit();
            } else {
              self.commit();
            }
          }
          else if c == '|' {
            self.next();
            
            if let Some('|') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpOr);
              self.commit();
            } else {
              return Err(self.error());
            }
          }
          else if c == '&' {
            self.next();
            
            if let Some('&') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpAnd);
              self.commit();
            } else {
              return Err(self.error()); 
            }
          }
          else if c == '<' { 
            self.new_token(TokenType::OpLs);
            self.next();
            
            if let Some('=') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpLsEq);
              self.commit();
            } else {
              self.commit();
            }
          }
          else if c == '>' { 
            self.new_token(TokenType::OpGt);
            self.next();
            
            if let Some('=') = self.peek_char() {
              self.next();
              self.new_token(TokenType::OpGtEq);
              self.commit();
            } else {
              self.commit();
            }
          }
          else if c == ' ' || c == '\t' || c == '\n' {
            self.next();
            self.reset();
          }
          else {
            return Err(self.error()); 
          }
        }
      }
    }

    self.new_token(TokenType::Eof);
    self.commit();
    
    Ok(&self.tokens)
  }

  fn cur_text(&mut self) -> &'a str { 
    let &(offset, _) = self.it.peek().unwrap_or(&(self.start, '\0'));
    
    &self.text[self.start..offset]
  }

  fn peek_char(&mut self) -> Option<char> {
    if let Some(&(_, ch)) = self.it.peek() {
      Some(ch)
    } else {
      None
    }
  }

  fn peek_pos(&mut self) -> Option<usize> {
    if let Some(&(pos, _)) = self.it.peek() {
      Some(pos)
    } else {
      None
    }
  }

  fn new_token(&mut self, t: TokenType) {
    self.token = Token::new(t, "", self.line, self.col);
  }
  
  fn commit(&mut self) {
    self.token.text = self.cur_text();
    self.tokens.push_back(self.token.clone());
    self.reset();
  }

  fn reset(&mut self) {
    self.token = Token::new_empty();
    self.start = self.peek_pos().unwrap_or(self.text.len()); 
  }

  fn next(&mut self) {
    if let Some('\n') = self.peek_char() {
      self.line += 1;
      self.col = 0; 
    } else {
      self.col += 1;
    }
    
    self.it.next();
  }

  fn error(&mut self) -> String {
    let ch = if let Some(ch) = self.peek_char() {
      ch.to_string()
    } else {
      "EOF".to_string()
    };
    return format!("Unknown character at line {} column {}: {}", self.line, self.col, ch); 
  }
}

