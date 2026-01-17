use crate::{ast::{ParseFrame, TreeElement}, block::BlockLevelAttribute, inline::data_builder::DataBuilder};

mod data_builder;
mod parse_inline;

pub fn parse(block_tree: Vec<crate::block::BlockLevelAttribute>) -> Vec<crate::ast::TreeElement> {
  let mut db = DataBuilder::new();

  let mut iters = vec![block_tree.into_iter()];

  while !iters.is_empty() {
    if let Some(block) = iters.last_mut().unwrap().next() {
      match block {
        BlockLevelAttribute::BlockQuote(children) => {
          db.push(ParseFrame::QuoteBlock);
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::TabView(children) => {
          db.push(ParseFrame::TabView);
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::Table(table) => {
          let mut res = vec![vec![]];

          for vc in table {
            for item in vc {
              res.last_mut().unwrap().push(crate::ast::table_cell::Cell {
                val: parse_inline::parse_inline(item.val),
                style: item.style,
                spanning: item.spanning,
              })
            }
          }

          db.add(TreeElement::Table(res));
        }

        BlockLevelAttribute::Tab { title, children } => {
          db.push(ParseFrame::Tab(title));
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::Inline(children) => {
          db.add(TreeElement::Paragraph(parse_inline::parse_inline(children)));
        }
      }
    } else {
      db.pop_and_merge();
    }
  }

  db.into()
}

