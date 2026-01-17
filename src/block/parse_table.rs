pub fn parse_table(buf: &mut Vec<crate::tokenizer::Token>) -> Vec<super::BlockLevelAttribute> {
  use crate::tokenizer::Token;
  let buf = std::mem::take(buf);

  let mut res: Vec<super::BlockLevelAttribute> = vec![];

  let mut table: Vec<Vec<super::table_cell::BlockCell>> = vec![];
  let mut spanning_count = 0;
  let mut is_table_line = false;
  let mut now_buf: Vec<crate::tokenizer::Token> = vec![];
  let mut is_last_newline = true;
  let mut recent_cell_style: Option<super::table_cell::Style> = None;

  for token in buf {
    match token {
      Token::CellSeparator(v) => {
        if is_last_newline {
          is_table_line = true;

          if table.is_empty() && !now_buf.is_empty() && is_last_newline { // 前までの要素を書き出す
            // 改行が入っているので除去する
            now_buf.pop().unwrap();

            res.push(super::BlockLevelAttribute::Inline(std::mem::take(&mut now_buf)));
          }

          table.push(vec![]);
        }

        if is_table_line {
          // 左にセルがあるならそれを書き出す.
          // もし直前がセパレータならnow_bufは空であるから, Spanningを消して良い
          if !now_buf.is_empty() && !is_last_newline {
            table.last_mut().unwrap().push(super::table_cell::BlockCell {
              val: std::mem::take(&mut now_buf),
              style: recent_cell_style,
              spanning: spanning_count.try_into().unwrap(),
            });
            spanning_count = 0;
          } else if !now_buf.is_empty() && is_last_newline {
            res.push(super::BlockLevelAttribute::Inline(std::mem::take(&mut now_buf)));
          }

          recent_cell_style = v;
          spanning_count+=1;
        } else {
          // ===== 文字として解釈する =====

          // 文字列を復元
          let mut txt = String::from("||");
          if let Token::CellSeparator(Some(style)) = token {
            txt += match style {
              super::table_cell::Style::LeftAligned => { "<" }
              super::table_cell::Style::RightAligned => { ">" }
              super::table_cell::Style::CenterAligned => { "=" }
              super::table_cell::Style::Title => { "~" }
            };
          }

          // バッファに書き込む
          if !now_buf.is_empty() {
            let v = now_buf.pop().unwrap();
            if let Token::Text(mut st) = v {
              st.push_str(&txt);
              now_buf.push(Token::Text(st))
            } else {
              now_buf.push(v);
              now_buf.push(Token::Text(txt));
            }
          } else {
            now_buf.push(Token::Text(txt));
          }
        }

        is_last_newline = false;
      }

      Token::NewLine => {
        // 通常終了処理
        if !is_table_line { // 通常処理であるか
          if is_last_newline { // ブロック終了
            if !now_buf.is_empty() {
              res.push(super::BlockLevelAttribute::Inline(std::mem::take(&mut now_buf)));
            }
          } else { // ブロック内改行
            now_buf.push(Token::NewLine);
          }
        } else { // 直前行がテーブル
          // テーブルの末尾をクリア
          now_buf.clear();
          spanning_count = 0;
        }

        is_last_newline = true;
        is_table_line = false;
      }

      _ => {
        if !table.is_empty() && !is_table_line {
          // テーブル終了処理
          res.push(super::BlockLevelAttribute::Table(std::mem::take(&mut table)));
          now_buf.clear();
          spanning_count = 0;
        }

        now_buf.push(token);

        is_last_newline = false;
      }
    };
  }

  // flush
  if !table.is_empty() { // table mode
    now_buf.clear();
    res.push(super::BlockLevelAttribute::Table(std::mem::take(&mut table)));
  } else if !now_buf.is_empty() { // normal mode
    res.push(super::BlockLevelAttribute::Inline(std::mem::take(&mut now_buf)));
  }

  res
}

mod tests {
  use crate::{block::{BlockLevelAttribute, table_cell::{BlockCell, Style}}, tokenizer::Token};
  use super::*;
  use crate::tokenizer;

  #[test]
  fn test_empty() {
    assert_eq!(parse_table(&mut vec![]), vec![]);
  }

  #[test]
  fn test_short_text() {
    assert_eq!(parse_table(&mut tokenizer::tokenize(String::from("Hello, World!"))), vec![
      BlockLevelAttribute::Inline(vec![
        Token::Text(String::from("Hello, World!"))
      ])
    ])
  }

  fn nz(v: usize) -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::try_from(v).unwrap()
  }

  #[test]
  fn test_table_single() {
    assert_eq!(parse_table(&mut tokenizer::tokenize(String::from("b\n|| a || b || c ||\na"))), vec![
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("b"))]),
      BlockLevelAttribute::Table(vec![
        vec![
          BlockCell { val: vec![Token::Text(String::from(" a "))], style: None, spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from(" b "))], style: None, spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from(" c "))], style: None, spanning: nz(1) },
        ]
      ]),
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("a"))]),
    ])
  }

  #[test]
  fn test_table_multi() {
    println!("debug: {:?}", tokenizer::tokenize(String::from("b\n||~ a ||~ b ||~ c ||  \n||< d ||> e||=f ||\ng")));
    assert_eq!(parse_table(&mut tokenizer::tokenize(String::from("b\n||~ a ||~ b ||~ c ||  \n||< d ||> e||=f ||\ng"))), vec![
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("b"))]),
      BlockLevelAttribute::Table(vec![
        vec![
          BlockCell { val: vec![Token::Text(String::from(" a "))], style: Some(Style::Title), spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from(" b "))], style: Some(Style::Title), spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from(" c "))], style: Some(Style::Title), spanning: nz(1) },
        ],
        vec![
          BlockCell { val: vec![Token::Text(String::from(" d "))], style: Some(Style::LeftAligned), spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from(" e"))], style: Some(Style::RightAligned), spanning: nz(1) },
          BlockCell { val: vec![Token::Text(String::from("f "))], style: Some(Style::CenterAligned), spanning: nz(1) },
        ]
      ]),
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("g"))]),
    ])
  }
}