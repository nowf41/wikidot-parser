use crate::tokenizer::Token;
use crate::ast::table_cell;

mod parse_table;
mod data_builder;

#[derive(PartialEq, Eq, Debug)]
pub enum BlockLevelAttribute {
  BlockQuote(Vec<BlockLevelAttribute>),
  Table(Vec<Vec<table_cell::Cell>>), // Inline以外中には入らないようにする必要がある.
  TabView(Vec<(String, Vec<BlockLevelAttribute>)>),

  Inline(Vec<crate::tokenizer::Token>), // トップレベルのInlineは段落を示す.
}

pub enum BlockLevelFrame {
  BlockQuote,
  TabView {
    tabs: Vec<(String, Vec<BlockLevelAttribute>)>,
  },
  Tab{title: String},
  // Table ... trailing element
  // Inline ... trailing element
}


pub fn parse(tokens: Vec<crate::tokenizer::Token>) -> Vec<BlockLevelAttribute> {
  let mut db = data_builder::DataBuilder::new();

  let mut is_last_newline = false;
  for token in tokens {
    match token {
      Token::BlockQuote(level) => {
        db.set_bq_depth(level.get());

        is_last_newline = false;
      }

      Token::ElementBegin { ref name, ref attributes } => {
        if is_last_newline {
          db.set_bq_depth(0);
        }

        match name.as_str() {
          "tabview" => {
            db.push(BlockLevelFrame::TabView { tabs: vec![] });
          }

          "tab" => {
            let mut title = String::new();
            for (key, value) in attributes {
              if key.is_empty() {
                if !title.is_empty() {
                  title.push(' ');
                }
                title += value;
              }
            }

            db.push(BlockLevelFrame::Tab { title });
          }

          &_ => {
            db.add_token(token);
          }
        }

        is_last_newline = false;
      }

      Token::ElementEnd(ref name) => {
        if is_last_newline {
          db.set_bq_depth(0);
        }

        match name.as_str() {
          "tabview" => {
            if let Some(BlockLevelFrame::TabView { tabs: _ }) = db.get_last_frame() {
              db.pop_and_merge();
            }
          }

          "tab" => {
            if let Some(BlockLevelFrame::Tab { title: _ }) = db.get_last_frame() {
              db.pop_and_merge();
            }
          }

          &_ => {
            db.add_token(token);
          }
        }

        is_last_newline = false;
      }

      Token::NewLine => {
        if is_last_newline {
          db.flush();
          db.set_bq_depth(0);
        } else {
          db.add_token(token);
        }

        is_last_newline = true;
      }

      _ => {
        if is_last_newline {
          db.set_bq_depth(0);
        }

        db.add_token(token);
      }
    }
  }

  db.get()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn nz(v: usize) -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::try_from(v).unwrap()
  }

  fn sf(st: &str) -> String {
    String::from(st)
  }

  #[test]
  fn test_empty() {
    assert_eq!(parse(vec![]), vec![]);
  }

  #[test]
  fn test_blockquote() {
    use crate::tokenizer::Token;
    assert_eq!(parse(vec![
      Token::BlockQuote(nz(1)), Token::Text(String::from("Hello,")), Token::NewLine,
      Token::BlockQuote(nz(2)), Token::Text(String::from("World!")), Token::NewLine
    ]), vec![
      BlockLevelAttribute::BlockQuote(vec![
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("Hello,"))]),
        BlockLevelAttribute::BlockQuote(vec![
          BlockLevelAttribute::Inline(vec![Token::Text(String::from("World!"))])
        ]),
      ])
    ]);
  }

  #[test]
  fn test_table_in_blocks() {
    use crate::tokenizer::tokenize;
    use crate::tokenizer::Token;
    // "a\n|| a || b ||\nc"
    let tokens = tokenize(String::from("a\n|| a || b ||\nc"));
    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("a"))]),
      BlockLevelAttribute::Table(vec![
        vec![
          crate::block::table_cell::Cell { val: vec![Token::Text(String::from(" a "))], style: None, spanning: nz(1) },
          crate::block::table_cell::Cell { val: vec![Token::Text(String::from(" b "))], style: None, spanning: nz(1) },
        ]
      ]),
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("c"))]),
    ]);
  }

  #[test]
  fn test_nested_closings() {
    use crate::tokenizer::Token;
    // depth 1 -> 2 -> 3 then close to 1 and continue
    let tokens = vec![
      Token::BlockQuote(nz(1)), Token::Text(String::from("L1")), Token::NewLine,
      Token::BlockQuote(nz(2)), Token::Text(String::from("L2")), Token::NewLine,
      Token::BlockQuote(nz(3)), Token::Text(String::from("L3")), Token::NewLine,
      Token::BlockQuote(nz(1)), Token::Text(String::from("After")), Token::NewLine,
    ];

    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::BlockQuote(vec![
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("L1"))]),
        BlockLevelAttribute::BlockQuote(vec![
          BlockLevelAttribute::Inline(vec![Token::Text(String::from("L2"))]),
          BlockLevelAttribute::BlockQuote(vec![
            BlockLevelAttribute::Inline(vec![Token::Text(String::from("L3"))])
          ]),
        ]),
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("After"))]),
      ])
    ]);
  }

  #[test]
  fn test_tabview() {
    use crate::tokenizer::Token;

    let tokens = vec![
      Token::ElementBegin { name: String::from("tabview"), attributes: vec![] },
      Token::ElementBegin { name: String::from("tab"), attributes: vec![(String::from(""), String::from("Tab 1"))] },
      Token::Text(String::from("txt 1")),
      Token::ElementEnd(String::from("tab")),
      Token::ElementBegin { name: String::from("tab"), attributes: vec![(String::from(""), String::from("Tab 2"))] },
      Token::Text(String::from("txt 2")),
      Token::ElementEnd(String::from("tab")),
      Token::ElementBegin { name: String::from("tab"), attributes: vec![(String::from(""), String::from("Tab 3"))] },
      Token::Text(String::from("txt 3")),
      Token::ElementEnd(String::from("tab")),
      Token::ElementEnd(String::from("tabview")),
    ];

    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::TabView(vec![
        (String::from("Tab 1"), vec![BlockLevelAttribute::Inline(vec![Token::Text(String::from("txt 1"))])]),
        (String::from("Tab 2"), vec![BlockLevelAttribute::Inline(vec![Token::Text(String::from("txt 2"))])]),
        (String::from("Tab 3"), vec![BlockLevelAttribute::Inline(vec![Token::Text(String::from("txt 3"))])]),
      ]),
    ]);
  }

  #[test]
  fn test_tabview_in_blockquote() {
    use crate::tokenizer::Token;

    let tokens = vec![
      Token::BlockQuote(nz(1)),
      Token::ElementBegin { name: String::from("tabview"), attributes: vec![] },
      Token::ElementBegin { name: String::from("tab"), attributes: vec![(String::from(""), String::from("Tab 1"))] },
      Token::Text(String::from("txt 1")),
      Token::ElementEnd(String::from("tab")),
      Token::ElementBegin { name: String::from("tab"), attributes: vec![(String::from(""), String::from("Tab 2"))] },
      Token::Text(String::from("txt 2")),
      Token::ElementEnd(String::from("tab")),
      Token::ElementEnd(String::from("tabview")),
    ];

    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::BlockQuote(vec![
        BlockLevelAttribute::TabView(vec![
          (String::from("Tab 1"), vec![BlockLevelAttribute::Inline(vec![Token::Text(String::from("txt 1"))])]),
          (String::from("Tab 2"), vec![BlockLevelAttribute::Inline(vec![Token::Text(String::from("txt 2"))])]),
        ]),
      ]),
    ]);
  }
}