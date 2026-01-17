use crate::tokenizer::Token;

use super::{BlockLevelAttribute, BlockLevelFrame};

pub struct DataBuilder {
  root: Vec<BlockLevelAttribute>,
  data: Vec<(BlockLevelFrame, Vec<BlockLevelAttribute>)>,
  buf: Vec<Token>,
  blockquote_depth_count: usize
}

impl DataBuilder {
  pub fn new() -> Self {
    Self {
      root: vec![],
      data: vec![],
      buf: vec![],
      blockquote_depth_count: 0
    }
  }

  fn pop(&mut self) -> Option<(BlockLevelFrame, Vec<BlockLevelAttribute>)> {
    let v = self.data.pop();
    if let Some(val) = &v && let BlockLevelFrame::BlockQuote = val.0 {
      self.blockquote_depth_count-=1;
    };

    v
  }

  pub fn flush(&mut self) {
    while let Some(Token::NewLine) = self.buf.last() {
      self.buf.pop();
    }
    if !self.buf.is_empty() {
      let target: &mut Vec<BlockLevelAttribute>;
      if let Some(pt) = self.data.last_mut() {
        target = &mut pt.1;
      } else {
        target = &mut self.root;
      }

      for v in super::parse_table::parse_table(&mut self.buf) {
        target.push(v);
      }
    }
  }

  pub fn pop_and_merge(&mut self) -> bool {
    self.flush();

    if let Some((now_frame, now_children)) = self.pop() {
      let push_target: &mut Vec<BlockLevelAttribute>;
      if let Some(ar) = self.data.last_mut() {
        push_target = &mut ar.1;
      } else {
        push_target = &mut self.root;
      }

      match now_frame {
        BlockLevelFrame::BlockQuote => {
          push_target.push(BlockLevelAttribute::BlockQuote(now_children));
        }

        BlockLevelFrame::TabView => {
          push_target.push(BlockLevelAttribute::TabView(now_children));
        }

        BlockLevelFrame::Tab { title } => {
          push_target.push(BlockLevelAttribute::Tab { title, children: now_children });
        }
      }
      true
    } else {
      false
    }
  }

  pub fn push(&mut self, frame: BlockLevelFrame) {
    self.flush();

    if let BlockLevelFrame::BlockQuote = &frame {
      self.blockquote_depth_count+=1;
    }
    self.data.push((frame, vec![]));
  }

  pub fn add(&mut self, data: BlockLevelAttribute) {
    self.flush();

    if let Some((_, target)) = self.data.last_mut() {
      target.push(data);
    }
  }

  pub fn add_token(&mut self, token: Token) {
    if self.buf.last().is_none_or(|v| *v == Token::NewLine) && token == Token::NewLine {
    } else {
      self.buf.push(token);
    }
  }

  fn stack_is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn set_bq_depth(&mut self, depth: usize) {
    if self.blockquote_depth_count > depth {
      while self.blockquote_depth_count > depth && !self.stack_is_empty() {
        self.pop_and_merge();
      }
    }

    if self.blockquote_depth_count < depth {
      while self.blockquote_depth_count < depth {
        self.push(BlockLevelFrame::BlockQuote);
      }
    }
  }

  pub fn get_last_frame(&self) -> Option<&BlockLevelFrame> {
    if let Some(v) = self.data.last() {
      Some(&v.0)
    } else {
      None
    }
  }

  pub fn get_last_frame_mut(&mut self) -> Option<&mut BlockLevelFrame> {
    if let Some(v) = self.data.last_mut() {
      Some(&mut v.0)
    } else {
      None
    }
  }

  pub fn get(mut self) -> Vec<BlockLevelAttribute> {
    self.flush();
    while self.pop_and_merge() {}
    self.root
  }
}