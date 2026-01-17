mod inline_builder;

use crate::ast;
use crate::tokenizer::Token;

pub fn parse_inline(tokens: Vec<crate::tokenizer::Token>) -> Vec<crate::ast::TreeElement> {
  let mut db = inline_builder::InlineBuilder::new();
  db.push(ast::ParseFrame::Paragraph);

  for token in tokens {
    if let Ok(frame) = token.clone().try_into() {
      db.switch_element(frame);
      continue;
    } else {
      match token {
        Token::Bold | Token::Italics | Token::Underline | Token::Strikethrough | Token::Superscript | Token::Subscript => {
          unreachable!() // handled above
        }

        Token::MonospacedOpen => {
          db.push(ast::ParseFrame::Monospaced);
        }

        Token::MonospacedClose => {
          db.close_element(ast::ParseFrameKind::Monospaced);
        }

        Token::ElementBegin { name, attributes } => {
          todo!();
        }

        Token::ElementEnd(name) => {
          todo!();
        }

        Token::ColoredBeginColorName(name) => {
          db.push(match name.as_str() {
            "aqua"    => ast::ParseFrame::Colored { red: 0x00, green: 0xFF, blue: 0xFF },
            "black"   => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0x00 },
            "blue"    => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0xFF },
            "fuchsia" => ast::ParseFrame::Colored { red: 0xFF, green: 0x00, blue: 0xFF }, // magenta
            "grey"    => ast::ParseFrame::Colored { red: 0x80, green: 0x80, blue: 0x80 },
            "green"   => ast::ParseFrame::Colored { red: 0x00, green: 0x80, blue: 0x00 },
            "lime"    => ast::ParseFrame::Colored { red: 0x00, green: 0xFF, blue: 0x00 },
            "maroon"  => ast::ParseFrame::Colored { red: 0x80, green: 0x00, blue: 0x00 },
            "navy"    => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0x80 },
            "olive"   => ast::ParseFrame::Colored { red: 0x80, green: 0x80, blue: 0x00 },
            "purple"  => ast::ParseFrame::Colored { red: 0x80, green: 0x00, blue: 0x80 },
            "red"     => ast::ParseFrame::Colored { red: 0xFF, green: 0x00, blue: 0x00 },
            "silver"  => ast::ParseFrame::Colored { red: 0xC0, green: 0xC0, blue: 0xC0 },
            "teal"    => ast::ParseFrame::Colored { red: 0x00, green: 0x80, blue: 0x80 },
            "white"   => ast::ParseFrame::Colored { red: 0xFF, green: 0xFF, blue: 0xFF },
            "yellow"  => ast::ParseFrame::Colored { red: 0xFF, green: 0xFF, blue: 0x00 },
            &_        => unreachable!(),
          });
        }

        Token::ColoredBeginColorCode(code) => {
          let chars: Vec<char> = code.chars().collect();

          let r = u8::from_str_radix(&chars[0..2].iter().collect::<String>(), 16).unwrap_or(0);
          let g = u8::from_str_radix(&chars[2..4].iter().collect::<String>(), 16).unwrap_or(0);
          let b = u8::from_str_radix(&chars[4..6].iter().collect::<String>(), 16).unwrap_or(0);

          db.push(ast::ParseFrame::Colored { red: r, green: g, blue: b });
        }

        Token::ColoredEnd => {
          db.close_element(ast::ParseFrameKind::Colored);
        }

        Token::NamedLink { link, name } => {
          db.add(ast::TreeElement::Link { href: ast::Url(link), open_in_new_tab: false, name });
        }

        Token::PageLink { link, name } => {
          db.add(ast::TreeElement::InternalLink { href: link, open_in_new_tab: false, name });
        }

        Token::BlockQuote(level) => {
          unreachable!(); // already handled in block parsing
        }

        Token::CellSeparator(_) => {
          unreachable!(); // already handled in block parsing
        }

        Token::NewLine => {
          db.add(ast::TreeElement::NewLine);
        }

        Token::Text(text) => {
          db.add(ast::TreeElement::Text(text));
        }
      }
    }
  }

  db.into()
}