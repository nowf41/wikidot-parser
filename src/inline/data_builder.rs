use crate::ast::{ParseFrame, TreeElement};

pub struct DataBuilder {
  root: Vec<TreeElement>,
  data: Vec<(ParseFrame, Vec<TreeElement>)>,
}

impl DataBuilder {
  pub fn new() -> Self {
    Self {
      root: vec![],
      data: vec![],
    }
  }

  pub fn pop_and_merge(&mut self) -> bool {
    if let Some((frame, elem)) = self.data.pop() {
      let push_target: &mut Vec<TreeElement>;
      if self.data.is_empty() {
        push_target = &mut self.root;
      } else {
        push_target = &mut self.data.last_mut().unwrap().1;
      }

      push_target.push(frame.into_tree_element(elem));

      true
    } else {
      false
    }
  }

  pub fn push(&mut self, fr: ParseFrame) {
    self.data.push((fr, vec![]));
  }

  pub fn add(&mut self, item: TreeElement) {
    if let Some((_, elem)) = self.data.last_mut() {
      elem.push(item);
    } else {
      self.root.push(item);
    }
  }

  pub fn last_frame_mut(&mut self) -> Option<&mut ParseFrame> {
    if let Some((fr, _)) = self.data.last_mut() {
      Some(fr)
    } else {
      None
    }
  }
}

impl From<DataBuilder> for Vec<TreeElement> {
  fn from(mut data: DataBuilder) -> Vec<TreeElement> {
    while data.pop_and_merge() {}
    data.root
  }
}