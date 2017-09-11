use syntax_tree::Visitor;
use syntax_tree::Node;
use syntax_tree::NodeType;
use frame_stack::FrameStackTree;

pub fn build_frame_stack(ast: &mut Node) -> FrameStackTree {
  let mut fstack = FrameStackTree::new();
  ast.visit(&mut LocalPass::new(&mut fstack));
  fstack.reset();
  ast.visit(&mut GlobalPass::new(&mut fstack));
  fstack.reset();

  fstack
}

struct LocalPass<'a> {
  fstack: &'a mut FrameStackTree
}

impl<'a> LocalPass<'a> {
  fn new(fstack: &mut FrameStackTree) -> LocalPass {
    LocalPass {
      fstack: fstack
    }
  }
}

impl<'a> Visitor for LocalPass<'a> {
  fn enter_var(&mut self, node: &mut Node) {
    let name = match node.body[0].type_ {
      NodeType::Symbol(ref s) => s,
      _ => panic!()
    };
    self.fstack.put_var(&name); 
  }

  fn enter_fun(&mut self, node: &mut Node) {
    self.fstack.add_child();
    self.fstack.enter();

    let args = &node.body[0].body;

    let frame = self.fstack.cur_frame();
    for arg in args.iter() {
      if let NodeType::Symbol(ref s) = arg.type_ {
        self.fstack.frames()[frame].var_offsets.insert(0, s.clone());
      }
    }
  }

  fn exit_fun(&mut self, _node: &mut Node) {
    self.fstack.exit();
  }
}

struct GlobalPass<'a> {
  fstack: &'a mut FrameStackTree
}

impl<'a> GlobalPass<'a> {
  fn new(fstack: &mut FrameStackTree) -> GlobalPass {
    GlobalPass {
      fstack: fstack
    }
  }
}

impl<'a> Visitor for GlobalPass<'a> {
  fn enter_assign(&mut self, node: &mut Node) {
    let name = match node.body[0].type_ {
      NodeType::Symbol(ref s) => s,
      _ => { return; }
    };

    if self.fstack.find_var(name).is_none() {
      self.fstack.put_var_global(&name); 
    }
  }

  fn enter_fun(&mut self, _node: &mut Node) {
    self.fstack.enter();
  }

  fn exit_fun(&mut self, _node: &mut Node) {
    self.fstack.exit();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokenizer::Tokenizer;
  use parser::Parser;
  use frame_stack::Frame;

  #[test]
  fn test_analyser() {
    let text = "var a = fn() {
      var b = 13;
      var c = fn() {
        var d = 12;
        var e = d + b;
        g1 = 1;
        return e;
      };
      g2 = 2;
      return c;
    }; var f = 1; g3 = 3;";
    let mut ast = Parser::new(Tokenizer::new(&text)
                          .tokenize().unwrap()).parse();
    let mut fstack = FrameStackTree::new();
    let frame_has_var = |f : &Frame, st| f.var_offsets.iter().find(|&s| s == st).is_some();

    {
      let mut local_pass = LocalPass::new(&mut fstack);
      ast.visit(&mut local_pass);
      
      assert_eq!(local_pass.fstack.frames().len(), 3);

      assert_eq!(local_pass.fstack.frames()[0].var_offsets.len(), 3);
      assert!(frame_has_var(&local_pass.fstack.frames()[0], "a"));
      assert!(frame_has_var(&local_pass.fstack.frames()[0], "f"));

      assert_eq!(local_pass.fstack.frames()[1].var_offsets.len(), 3);
      assert!(frame_has_var(&local_pass.fstack.frames()[1], "b"));
      assert!(frame_has_var(&local_pass.fstack.frames()[1], "c"));

      assert_eq!(local_pass.fstack.frames()[2].var_offsets.len(), 3);
      assert!(frame_has_var(&local_pass.fstack.frames()[2], "d"));
      assert!(frame_has_var(&local_pass.fstack.frames()[2], "e"));
    }

    fstack.reset();
    
    {
      let mut global_pass = GlobalPass::new(&mut fstack);
      ast.visit(&mut global_pass);
      assert_eq!(global_pass.fstack.frames()[0].var_offsets.len(), 6);
      assert!(frame_has_var(&global_pass.fstack.frames()[0], "g1"));
      assert!(frame_has_var(&global_pass.fstack.frames()[0], "g2"));
      assert!(frame_has_var(&global_pass.fstack.frames()[0], "g3"));
    }
  }
}

