use crate::ast;

pub struct InlineBuilder {
  root: Vec<ast::TreeElement>,
  data: Vec<(ast::ParseFrame, Vec<ast::TreeElement>)>,
}

impl InlineBuilder {
  pub fn new() -> Self {
    Self {
      root: vec![],
      data: vec![],
    }
  }

  pub fn pop_and_merge(&mut self) -> Option<ast::ParseFrame> {
    if let Some((frame, container)) = self.data.pop() {
      if let Some((_, parent_container)) = self.data.last_mut() {
        parent_container.push(frame.clone().into_tree_element(container));
      } else {
        self.root.push(frame.clone().into_tree_element(container));
      }

      Some(frame)
    } else {
      None
    }
  }

  pub fn close_element(&mut self, kind: ast::ParseFrameKind) -> bool {
    let mut frame_to_reopen: Vec<ast::ParseFrame> = vec![]; // Frames that need to be reopened

    let mut reached = false;
    while let Some(frame) = self.pop_and_merge() {
      if frame.get_kind() == kind {
        reached = true;
        break;
      } else {
        frame_to_reopen.push(frame);
      }
    }

    for frame in frame_to_reopen {
      self.push(frame);
    }

    reached
  }

  pub fn switch_element(&mut self, param_frame: ast::ParseFrame) {
    for (frame, _) in &self.data {
      if frame.get_kind() == param_frame.get_kind() {
        self.close_element(param_frame.get_kind()); // if found: open
        return;
      }
    }

    self.push(param_frame); // if not found: close
  }

  pub fn push(&mut self, frame: ast::ParseFrame) {
    self.data.push((frame, vec![]));
  }

  pub fn add(&mut self, element: ast::TreeElement) {
    if let Some((_, container)) = self.data.last_mut() {
      container.push(element);
    } else {
      self.root.push(element);
    }
  }
}

impl From<InlineBuilder> for Vec<ast::TreeElement> {
  fn from(mut builder: InlineBuilder) -> Vec<ast::TreeElement> {
    while let Some(_) = builder.pop_and_merge() {}
    builder.root
  }
}