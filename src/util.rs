use syntax_tree::Visitor;
use syntax_tree::Node;

pub struct GraphvizVisitor {
  text: String
}

impl GraphvizVisitor {
  pub fn new() -> GraphvizVisitor {
    GraphvizVisitor {
      text: String::new()
    }
  }

  pub fn begin(&mut self) {
    self.text += "digraph {\n";
    self.text += "\trankdir = LR;\n";
    self.text += "\tnode[shape=box fontname=\"Monospace\"];\n";
  }

  pub fn end(&mut self) {
    self.text += "}\n";
  }

  pub fn text(&self) -> String { 
    self.text.clone()
  }
}

impl Visitor for GraphvizVisitor {
  fn visit(&mut self, node: &mut Node) {
    let this_id = node as *const Node;
    
    let node_type = format!("{:?}", node.type_).replace("\"", "\\\"");
    self.text += &format!("\tnode{}[label=\"{}\"]\n", this_id as usize, &node_type); 

    for ch in node.body.iter() {
      let child_id = ch as *const Node;
      self.text += &format!("\tnode{} -> node{}\n", this_id as usize, child_id as usize);
    }
  }
}

