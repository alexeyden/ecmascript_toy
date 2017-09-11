pub struct Frame {
  pub var_offsets: Vec<String>
}

impl Frame {
  pub fn new() -> Frame {
    Frame {
      var_offsets: vec![ "this".to_string() ]
    }
  }
}

#[derive(PartialEq, Debug)]
struct Link { children: Vec<usize>, parent: usize }

pub struct VarDescr {
  pub frame_offset: usize,
  pub var_offset: usize,
  pub frame_id: usize 
}

pub struct FrameStackTree {
  frames: Vec<Frame>,
  links: Vec<Link>,
  cur_frame: usize,
  next_frame: usize
}

impl FrameStackTree {
  pub fn new() -> FrameStackTree {
    FrameStackTree {
      frames: vec![ Frame::new() ],
      links: vec![ Link { children: vec![], parent: 0 } ],
      cur_frame: 0,
      next_frame: 1
    }
  }

  pub fn root_frame(&mut self) -> &mut Frame {
   &mut self.frames[0]
  }

  pub fn cur_frame(&self) -> usize {
    self.cur_frame
  }

  pub fn frames(&mut self) -> &mut Vec<Frame> {
    &mut self.frames
  }

  pub fn reset(&mut self) {
    self.cur_frame = 0;
    self.next_frame = 1;
  }

  pub fn parents(&self) -> Vec<u32> {
    let mut parents : Vec<u32> = vec![];

    let mut cur = self.cur_frame;
    loop {
      let parent = self.links[cur].parent;

      if parent == cur {
        break;
      } else {
        parents.push(parent as u32);
        cur = parent;
      }
    }
    return parents;
  }

  pub fn enter(&mut self) {
    self.cur_frame = self.next_frame;
    self.next_frame = *self.links[self.next_frame].children.get(0).unwrap_or(&0);
  }

  pub fn exit(&mut self) {
    let parent = self.links[self.cur_frame].parent;
    let next = self.links[parent].children.iter().position(|&x| x == self.cur_frame).unwrap() + 1;
    self.next_frame = *self.links[parent].children.get(next).unwrap_or(&0);
    self.cur_frame = parent;
  }

  pub fn add_child(&mut self) {
    self.frames.push(Frame::new());
    self.links.push(Link { children: vec![], parent: self.cur_frame });
    let new = self.links.len() - 1;
    self.links[self.cur_frame].children.push(new);
    self.next_frame = new;
  }

  pub fn find_var(&mut self, name: &String) -> Option<VarDescr>
  {
    let mut frame_offset = 0;
    let mut frame = self.cur_frame;
    let mut var_offset;

    loop {
      var_offset = self.frames[frame].var_offsets.iter()
        .position(|n| n == name);

      let is_root = self.links[frame].parent == frame;
      let is_found = var_offset.is_some();
      if is_found || is_root { break; }

      frame = self.links[frame].parent;
      frame_offset += 1;
    }

    if let Some(offset) = var_offset {
      Some(VarDescr {
        frame_offset: frame_offset,
        var_offset: offset,
        frame_id: frame
      })
    } else { None }
  }

  pub fn put_var(&mut self, name: &String) {
    let index = self.frames[self.cur_frame].var_offsets.len() as u32;
    let mut offsets = &mut self.frames[self.cur_frame].var_offsets;
    if offsets.iter().find(|&x| x == name).is_none() {
      offsets.insert(index as usize, name.clone());
    }
  }

  pub fn put_var_global(&mut self, name: &String) {
    let index = self.frames[0].var_offsets.len() as u32;
    let offsets = &mut self.frames[0].var_offsets;
    if offsets.iter().find(|&x| x == name).is_none() {
      offsets.insert(index as usize, name.clone());
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_frame_stack() {
    /* fn a1() {      // 1
         fn b1() {}   // 2
         fn b2() {    // 3
          fn c1() {}  // 4
          fn c2() {}  // 5
         } 
         fn b3() {    // 6
          fn c1() {}  // 7
         }
       }
     */
    
    let mut fstack = FrameStackTree::new();
    fstack.add_child(); fstack.enter(); // a1
    fstack.add_child(); fstack.enter(); fstack.exit(); // b1

    assert_eq!(fstack.links, [
      Link {children: vec![1], parent: 0}, // root
      Link {children: vec![2], parent: 0}, // a1
      Link {children: vec![],  parent: 1}, // b1
    ]);
    assert_eq!(fstack.cur_frame, 1);
    assert_eq!(fstack.next_frame, 0);
    
    fstack.add_child(); // b2
    fstack.enter(); // b2
    fstack.add_child(); fstack.enter(); fstack.exit(); // c1
    fstack.add_child(); fstack.enter(); fstack.exit(); // c2
    fstack.exit(); // b2

    assert_eq!(fstack.links, [
      Link {children: vec![1],   parent: 0}, // root
      Link {children: vec![2,3], parent: 0}, // a1
      Link {children: vec![],    parent: 1}, // b1
      Link {children: vec![4,5], parent: 1}, // b2
      Link {children: vec![],    parent: 3}, // c1
      Link {children: vec![],    parent: 3}, // c2
    ]);
    assert_eq!(fstack.cur_frame, 1);
    assert_eq!(fstack.next_frame, 0);

    fstack.add_child(); // b3
    fstack.enter(); // b3
    fstack.add_child(); fstack.enter(); fstack.exit(); // c1
    fstack.exit(); // b3
    fstack.exit(); // a1

    assert_eq!(fstack.links, [
      Link {children: vec![1],     parent: 0}, // root
      Link {children: vec![2,3,6], parent: 0}, // a1
      Link {children: vec![],      parent: 1}, // b1
      Link {children: vec![4,5],   parent: 1}, // b2
      Link {children: vec![],      parent: 3}, // c1
      Link {children: vec![],      parent: 3}, // c2
      Link {children: vec![7],     parent: 1}, // b3
      Link {children: vec![],      parent: 6}, // c1
    ]);
    assert_eq!(fstack.cur_frame, 0);
    assert_eq!(fstack.next_frame, 0);

    fstack.reset();
    fstack.enter();
    fstack.enter();
    assert_eq!(fstack.cur_frame, 2);

    fstack.exit();
    fstack.enter();
    fstack.enter(); fstack.exit();
    assert_eq!(fstack.next_frame, 5);
    fstack.enter(); fstack.exit();
    assert_eq!(fstack.cur_frame, 3);
    fstack.exit();
    assert_eq!(fstack.next_frame, 6);
    fstack.enter();
    assert_eq!(fstack.next_frame, 7);
  }
}
