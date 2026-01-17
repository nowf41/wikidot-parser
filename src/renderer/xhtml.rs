mod builder;

pub fn render(ast: Vec<crate::ast::TreeElement>) -> String {
  use crate::ast::TreeElement;

  let mut res = builder::Builder::new();

  res.open(String::from("html"), vec![]);
  res.open(String::from("head"), vec![]);
  res.insert(String::from("meta"), vec![("charset", "UTF-8")]);
  res.insert(String::from("meta"), vec![("name", "viewport"), ("content", "width=device-width, initial-scale=1")]);
  res.close(); // </head>
  res.open(String::from("body"), vec![]);

  let mut iters = vec![ast.into_iter()];

  while !iters.is_empty() {
    if let Some(v) = iters.last_mut().unwrap().next() {
      match v {
        TreeElement::Paragraph(children) => {
          res.open(String::from("p"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Text(text) => {
          res.write(&text);
        }

        TreeElement::Bold(children) => {
          res.open(String::from("strong"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Italics(children) => {
          res.open(String::from("i"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Underline(children) => {
          res.open(String::from("span"), vec![("style", "text-decoration: underline")]);
          iters.push(children.into_iter());
        }

        TreeElement::Strikethrough(children) => {
          res.open(String::from("s"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Monospaced(children) => {
          res.open(String::from("code"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Superscript(children) => {
          res.open(String::from("sup"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Subscript(children) => {
          res.open(String::from("sub"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Colored{red, green, blue, children} => {
          res.open(String::from("span"), vec![("style", &format!("color: rgb({}, {}, {})", red, green, blue))]);
          iters.push(children.into_iter());
        }

        TreeElement::Size { scale, children } => {
          res.open(String::from("span"), vec![("style", &format!("font-size: {}", scale.as_string()))]);
          iters.push(children.into_iter());
        }

        TreeElement::Link { href, open_in_new_tab, name } => {
          let mut attrs = vec![("href", &href.0 as &str)];
          if open_in_new_tab {
            attrs.push(("target", "_blank"));
            attrs.push(("rel", "noopener noreferrer"));
          }
          res.open(String::from("a"), attrs);
          res.write(&name);
          res.close()
        }

        TreeElement::InternalLink { href, open_in_new_tab, name } => {
          todo!();
        }

        TreeElement::Collapsible(children) => {
          res.open(String::from("details"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Footnote{id, children} => {
          todo!();
        }

        TreeElement::QuoteBlock(children) => {
          res.open(String::from("blockquote"), vec![]);
          iters.push(children.into_iter());
        }

        TreeElement::Iframe(raw) => { // TODO size?
          res.open(String::from("iframe"), vec![("srcdoc", &raw)]);
          res.close();
        }

        TreeElement::Tab{title, children} => {

        }

        TreeElement::TabView(_) => {
          todo!()
        }

        TreeElement::Table(rows) => {
          res.open(String::from("table"), vec![]);
          for row in rows {
            res.open(String::from("tr"), vec![]);
            for cell in row {
              todo!();
            }
            res.close();
          }
          res.close();
        }

        TreeElement::NewLine => {
          res.insert(String::from("br"), vec![]);
        }

        TreeElement::HtmlElement { tag, property, children } => {
          let attrs: Vec<(&str, &str)> = property.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
          res.open(tag, attrs);
          iters.push(children.into_iter());
        }
      }
    } else {
      if iters.len() > 1 {res.close()}; // don't close root
      iters.pop();
    }
  }

  res.close(); // </body>
  res.close(); // </html>

  res.into()
}