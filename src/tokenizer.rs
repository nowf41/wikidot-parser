// According to https://scp-wiki.wikidot.com/wiki-syntax
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  Bold, // **
  Italics, // // (double-slash)
  Underline, // __
  Strikethrough, // --
  MonospacedOpen, // {{
  MonospacedClose, // }}
  SuperScript, // ^^
  SubScript, // ,,
  ElementBegin{name: String, attributes: Vec<(String, String)>}, // [[span style="color:red"]]
  ElementEnd(String), // [[/span]]
  ColoredBeginColorCode(String), // ##color|
  ColoredBeginColorName(&'static str),
  ColoredEnd, // ##
  EscapeParsing, // @@
  NamedLink{link: String, name: String},
  PageLink{link: String, name: String},
  Asterisk, // * (sometimes mean – target="_blank")
  BlockQuote(u8), // >>>> max nested is 255
  CellSeparator, // ||
  TableHeaderCellSeparator, // ||~

  Text(String)
}

struct TokenData {
  res: Vec<Token>,
  buf: String,
}

impl TokenData {
  fn new() -> Self {
    Self {
      res: vec![],
      buf: String::new(),
    }
  }

  fn add_char(&mut self, c: char) {
    self.buf.push(c);
  }

  fn flush(&mut self) {
    if !self.buf.is_empty() {
      let text = std::mem::take(&mut self.buf);
      self.res.push(Token::Text(text));
    }
  }

  fn flush_and_add_token(&mut self, t: Token) {
    self.flush();
    self.res.push(t);
  }

  fn get_value(self) -> (Vec<Token>, String) {
    let mut this = self;
    this.flush();
    (this.res, this.buf)
  }
}

fn is_next_eq(at: usize, v: &Vec<char>, c: char) -> bool {
  if at+1 >= v.len() {
    false
  } else {
    v[at+1] == c
  }
}

fn get_unescaped_string(s: &[char]) -> String {
  let mut target_str: String = String::new(); // エスケープを取り除かれた文字列
  let mut ignore_next = false;
  for j in 0..s.len() {
    if ignore_next {
      target_str.push(s[j]);
      ignore_next = false;
    } else {
      if s[j] == '\\' {
        ignore_next = true;
      } else {
        target_str.push(s[j]);
      }
    }
  }
  target_str
}

// TODO \n|の処理を書く
pub fn tokenize(s: &str) -> Vec<Token> {
  let mut data: TokenData = TokenData::new();

  let chars: Vec<char> = s.chars().collect();

  let mut is_escaping_parse = false;

  // TODO: optimize by making this static and changing type to HashMap
  let tokenize_if_double: Vec<(char, Token)> = vec![
    ('*', Token::Bold),
    ('/', Token::Italics),
    ('_', Token::Underline),
    ('-', Token::Strikethrough),
    ('{', Token::MonospacedOpen),
    ('}', Token::MonospacedClose),
    ('^', Token::SuperScript),
    (',', Token::SubScript),
    ('@', Token::EscapeParsing),
    ('|', Token::CellSeparator),
  ];

  let mut i = 0;
  'chars_loop: while i < chars.len() {
    // check escape
    if chars[i] == '@' {
      if is_next_eq(i, &chars, '@') {
        data.flush_and_add_token(Token::EscapeParsing);
        i += 2;
        is_escaping_parse = !is_escaping_parse;
        continue 'chars_loop;
      }
    }

    if is_escaping_parse {
      data.add_char(chars[i]);
      i += 1;
      continue 'chars_loop;
    }

    for (c, t) in &tokenize_if_double {
      if chars[i] == *c {
        if is_next_eq(i, &chars, *c) {
          data.flush_and_add_token(t.clone());
          i += 2;
          continue 'chars_loop;
        }
      }
    }

    match chars[i] {
      '[' => 'square_brace: {
        if is_next_eq(i, &chars, '[') {
          if is_next_eq(i+1, &chars, '[') {
            let mut elem_specifier_len = 0;
            while i+3+elem_specifier_len < chars.len() { // i+3+elem_specifier_lenに本体を伸ばせるかを見る
              if chars[i+2+elem_specifier_len] != '\\' && chars[i+3+elem_specifier_len] == ']' && is_next_eq(i+3+elem_specifier_len, &chars, ']') && is_next_eq(i+3+elem_specifier_len+1, &chars, ']') {
                break;
              }
              if chars[i+3+elem_specifier_len] == '\n' {
                break 'square_brace;
              }
              elem_specifier_len+=1;
            };

            let target_str: String = get_unescaped_string(&chars[i+3..i+3+elem_specifier_len]);

            if target_str.contains('|') {
              let v: (&str, &str) = target_str.split_once('|').unwrap();

              data.flush_and_add_token(Token::PageLink { link: String::from(v.0), name: String::from(v.1) });
            } else {
              data.flush_and_add_token(Token::PageLink { link: String::from(target_str), name: String::from("") });
            }

            i += 3 + elem_specifier_len + 3;
            continue 'chars_loop;
          } else {
            // elem_begin
            let mut elem_specifier_len = 0;
            while i+2+elem_specifier_len < chars.len() { // i+2+elem_specifier_lenに本体を伸ばせるかを見る
              if chars[i+1+elem_specifier_len] != '\\' && chars[i+2+elem_specifier_len] == ']' && is_next_eq(i+2+elem_specifier_len, &chars, ']') {
                break;
              }
              if chars[i+2+elem_specifier_len] == '\n' {
                break 'square_brace;
              }
              elem_specifier_len+=1;
            };

            let target_str: String = get_unescaped_string(&chars[i+2..i+2+elem_specifier_len]);

            if target_str.starts_with("/") {
              data.flush_and_add_token(Token::ElementEnd((&target_str[1..]).into()));
            } else {
              let mut name = String::new();
              let mut attributes: Vec<(String, String)> = vec![];

              for (at, v) in target_str.split_whitespace().enumerate() {
                if at == 0 {
                  name = String::from(v);
                }
                else {
                  if v.contains('=') {
                    let v: Vec<_> = v.splitn(2, '=').collect();
                    if v[0].len() > 0 && v[1].len() > 2 {
                      attributes.push((String::from(v[0]), String::from(&v[1][1..(v[1].len()-1)])));
                    }
                  } else {
                    attributes.push((String::from(""), String::from(v)));
                  }
                }
              }

              data.flush_and_add_token(Token::ElementBegin { name, attributes });
            }
            i += 2 + elem_specifier_len + 2;
            continue 'chars_loop;
          }
        } else {
          let mut elem_specifier_len = 0;

          while i+1+elem_specifier_len < chars.len() { // i+1+elem_specifier_lenに本体を伸ばせるかを見る
            if chars[i+elem_specifier_len] != '\\' && chars[i+1+elem_specifier_len] == ']' {
              break;
            }
            if chars[i+1+elem_specifier_len] == '\n' {
              break 'square_brace;
            }
            elem_specifier_len+=1;
          };

          let target_str = get_unescaped_string(&chars[i+1..i+1+elem_specifier_len]);

          if let Some(v) = target_str.split_once(" ") {
            data.flush_and_add_token(Token::NamedLink {
              link: String::from(v.0),
              name: String::from(v.1),
            });

            i += 1 + elem_specifier_len + 1;
            continue 'chars_loop;
          }
        }
      }

      // TODO: いくつかの他の記号に対応
      '|' => {
        if is_next_eq(i, &chars, '|') {
          if is_next_eq(i+1, &chars, '~') {
            data.flush_and_add_token(Token::TableHeaderCellSeparator);
            i += 2;
          } else {
            data.flush_and_add_token(Token::CellSeparator);
            i += 1;
          }
        }
      }

      '*' => {
        data.flush_and_add_token(Token::Asterisk);
      }

      '\\' => {
        if i+1 >= chars.len() || chars[i+1] == '\n' {
          data.add_char('\n');
          i += 1;
        } else {
          data.add_char(chars[i+1]);
          i += 1;
        }
      }

      '>' => {
        if i == 0 || chars[i-1] == '\n' {
          let mut level: usize = 1;
          while level < u8::MAX.into() && is_next_eq(level - 1 + i, &chars, '>') {
            level += 1;
          }
          if i+level >= chars.len() || chars[i+level] == ' ' {
            data.flush_and_add_token(Token::BlockQuote(level.try_into().unwrap())); // never overflows
            i += level - 1;
          }
        }
      }

      '#' => 'sharp_match: {
        if is_next_eq(i, &chars, '#') {
          // color code
          if chars.len() > i+8 {
            let mut ok = true;
            for j in 2..8 {
              let v = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'A', 'b', 'B', 'c', 'C', 'd', 'D', 'e', 'E', 'f', 'F'];
              if !v.contains(&chars[i+j]) {
                ok = false;
              }
            }
            if chars[i+8] != '|' { ok = false; }

            if ok {
              data.flush_and_add_token(Token::ColoredBeginColorCode((&chars[i+2..i+8]).iter().collect()));
              i += 2 + 6 + 1 - 1;
              break 'sharp_match;
            }
          }

          let wikidot_preset_colors = vec![
            "aqua",
            "black",
            "blue",
            "fuchsia",
            "grey",
            "green",
            "lime",
            "maroon",
            "navy",
            "olive",
            "purple",
            "red",
            "silver",
            "teal",
            "white",
            "yellow",
          ];

          let s: String = (&chars[i+2..std::cmp::min(i+2+8, chars.len())]).iter().collect();

          for wikidot_preset_color_string in wikidot_preset_colors {
            if s.starts_with(wikidot_preset_color_string) && s.chars().nth(wikidot_preset_color_string.len()) == Some('|') {
              data.flush_and_add_token(Token::ColoredBeginColorName(wikidot_preset_color_string));
              i += 2 + wikidot_preset_color_string.len() + 1 - 1;
              break 'sharp_match;
            }
          }

          data.flush_and_add_token(Token::ColoredEnd);
          i+=1;
        }
      }

      _ => {
        data.add_char(chars[i]);
      }
    };

    i += 1;
  }

  data.get_value().0
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_empty() {
    assert_eq!(tokenize(""), vec![]);
  }

  #[test]
  fn test_bold() {
    assert_eq!(tokenize("**bolded_string**"), vec![
      Token::Bold,
      Token::Text(String::from("bolded_string")),
      Token::Bold,
    ]);
  }

  #[test]
  fn test_italics() {
    assert_eq!(tokenize("//Italic text//"), vec![
      Token::Italics,
      Token::Text(String::from("Italic text")),
      Token::Italics,
    ]);
  }

  #[test]
  fn test_underline() {
    assert_eq!(tokenize("__Underlined text__"), vec![
      Token::Underline,
      Token::Text(String::from("Underlined text")),
      Token::Underline,
    ]);
  }

  #[test]
  fn test_strikethrough() {
    assert_eq!(tokenize("--Outdated Containment Procedure--"), vec![
      Token::Strikethrough,
      Token::Text(String::from("Outdated Containment Procedure")),
      Token::Strikethrough,
    ]);
  }

  #[test]
  fn test_monospaced() {
    assert_eq!(tokenize("{{Monospaced Text}}"), vec![
      Token::MonospacedOpen,
      Token::Text(String::from("Monospaced Text")),
      Token::MonospacedClose,
    ]);
  }

  #[test]
  fn test_superscript() {
    assert_eq!(tokenize("Super^^scripted^^text"), vec![
      Token::Text(String::from("Super")),
      Token::SuperScript,
      Token::Text(String::from("scripted")),
      Token::SuperScript,
      Token::Text(String::from("text")),
    ]);
  }

  #[test]
  fn test_subscript() {
    assert_eq!(tokenize("Sub,,scripted,,text"), vec![
      Token::Text(String::from("Sub")),
      Token::SubScript,
      Token::Text(String::from("scripted")),
      Token::SubScript,
      Token::Text(String::from("text")),
    ]);
  }

  #[test]
  fn test_elementbegin_and_elementend() {
    assert_eq!(tokenize(r#"aaa[[span id="box" checked]]Hey![[/span]]bbb"#), vec![
      Token::Text(String::from("aaa")),
      Token::ElementBegin {
        name: String::from("span"),
        attributes: vec![
          (String::from("id"), String::from("box")),
          (String::from(""), String::from("checked"))
        ],
      },
      Token::Text(String::from("Hey!")),
      Token::ElementEnd(String::from("span")),
      Token::Text(String::from("bbb")),
    ])
  }

  #[test]
  fn test_colored_colorcode() {
    assert_eq!(tokenize("bbb##ff00ff|Material Error##aaa"), vec![
      Token::Text(String::from("bbb")),
      Token::ColoredBeginColorCode(String::from("ff00ff")),
      Token::Text(String::from("Material Error")),
      Token::ColoredEnd,
      Token::Text(String::from("aaa")),
    ])
  }

  #[test]
  fn test_colored_colorname() {
    assert_eq!(tokenize("bbb##green|Test Passed##aaa"), vec![
      Token::Text(String::from("bbb")),
      Token::ColoredBeginColorName("green"),
      Token::Text(String::from("Test Passed")),
      Token::ColoredEnd,
      Token::Text(String::from("aaa")),
    ]);
  }

  #[test]
  fn test_namedlink() {
    assert_eq!(tokenize("[https://example.com example link]aa"), vec![
      Token::NamedLink { link: String::from("https://example.com"), name: String::from("example link") },
      Token::Text(String::from("aa")),
    ])
  }

  #[test]
  fn test_pagelink() {
    assert_eq!(tokenize("[[[example|hello]]]"), vec![
      Token::PageLink { link: String::from("example"), name: String::from("hello") },
    ])
  }

  #[test]
  fn test_asterisk() {
    assert_eq!(tokenize("hey*ho"), vec![
      Token::Text(String::from("hey")),
      Token::Asterisk,
      Token::Text(String::from("ho")),
    ])
  }

  #[test]
  fn test_quoteblock() {
    assert_eq!(tokenize("> One\n>> Two\n>> Three\n> Four\nFive"), vec![
      Token::BlockQuote(1), Token::Text(String::from("One\n")),
      Token::BlockQuote(2), Token::Text(String::from("Two\n")),
      Token::BlockQuote(2), Token::Text(String::from("Three\n")),
      Token::BlockQuote(1), Token::Text(String::from("Four\nFive")),
    ]);
  }

  #[test]
  fn test_escape_parsing() {
    assert_eq!(tokenize("@@**Should not be bolded**@@"), vec![
      Token::EscapeParsing,
      Token::Text(String::from("**Should not be bolded**")),
      Token::EscapeParsing,
    ]);
  }
}
