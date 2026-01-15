#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSize(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikidotColor {
  Aqua,
  Black,
  Blue,
  Fuchsia,
  Grey,
  Green,
  Lime,
  Maroon,
  Navy,
  Olive,
  Purple,
  Red,
  Silver,
  Teal,
  White,
  Yellow,
}

impl WikidotColor {
  pub fn from(value: &str) -> Option<Self> {
    match value.trim().to_lowercase().as_str() {
      "aqua" => { Some(Self::Aqua) },
      "black" => { Some(Self::Black) },
      "blue" => { Some(Self::Blue) },
      "fuchsia" => { Some(Self::Fuchsia) },
      "grey" => { Some(Self::Grey) },
      "green" => { Some(Self::Green) },
      "lime" => { Some(Self::Lime) },
      "maroon" => { Some(Self::Maroon) },
      "navy" => { Some(Self::Navy) },
      "olive" => { Some(Self::Olive) },
      "purple" => { Some(Self::Purple) },
      "red" => { Some(Self::Red) },
      "silver" => { Some(Self::Silver) },
      "teal" => { Some(Self::Teal) },
      "white" => { Some(Self::White) },
      "yellow" => { Some(Self::Yellow) },
      _ => { None }
    }
  }
}

pub mod table_cell {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Style {
    LeftAligned,
    RightAligned,
    CenterAligned,
    Title,
  }

  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct Cell {
    pub val: Vec<crate::tokenizer::Token>,
    pub style: Option<Style>,
    pub spanning: std::num::NonZeroUsize,
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeElement {
  Text(String),
  Bold(Vec<TreeElement>),
  Italics(Vec<TreeElement>),
  Underline(Vec<TreeElement>),
  Strikethrough(Vec<TreeElement>),
  Monospaced(Vec<TreeElement>),
  Superscript(Vec<TreeElement>),
  Subscript(Vec<TreeElement>),
  Colored{color: WikidotColor, children: Vec<TreeElement>},
  Size{scale: CssSize, children: Vec<TreeElement>}, // scaleは有効なCSS値
  Link{href: Url, open_in_new_tab: bool},
  InternalLink{href: String, open_in_new_tab: bool},
  Collapsible(Vec<TreeElement>),
  Footnote{id: u32, children: Vec<TreeElement>}, // idは構文解析時に自動的に生成
  QuoteBlock(Vec<TreeElement>),
  Iframe(String), // the value is raw HTML element string
  TabView(Vec<(String, Vec<TreeElement>)>),
  Table(Vec<Vec<table_cell::Cell>>),

  HtmlElement{tag: String, property: Vec<(String, String)>, children: Vec<TreeElement>},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFrame {
  Bold,
  Italics,
  Underline,
  Strikethrough,
  Monospaced,
  Superscript,
  Subscript,
  Colored{color: WikidotColor},
  Size{scale: CssSize},
  // Link does not contain children
  // InternalLink does not contain children
  Collapsible,
  Footnote{id: u32},
  QuoteBlock,
  // Iframe is a single element. The values are written in HTML and they won't be parsed.
  TabView(Vec<(String, Vec<TreeElement>)>), // Tabs will be joined to this (this is a special procession)
  // Table does not contain TreeElement children

  HtmlElement{tag: String, property: Vec<(String, String)>}, // should be filtered by its tag
}

impl ParseFrame {
  pub fn into_tree_element(self, children: Vec<TreeElement>) -> TreeElement {
    match self {
      ParseFrame::Bold => TreeElement::Bold(children),
      ParseFrame::Italics => TreeElement::Italics(children),
      ParseFrame::Underline => TreeElement::Underline(children),
      ParseFrame::Strikethrough => TreeElement::Strikethrough(children),
      ParseFrame::Monospaced => TreeElement::Monospaced(children),
      ParseFrame::Superscript => TreeElement::Superscript(children),
      ParseFrame::Subscript => TreeElement::Subscript(children),
      ParseFrame::Colored{color} => TreeElement::Colored{color, children},
      ParseFrame::Size{scale} => TreeElement::Size{scale, children},
      ParseFrame::Collapsible => TreeElement::Collapsible(children),
      ParseFrame::Footnote{id} => TreeElement::Footnote{id, children},
      ParseFrame::QuoteBlock => TreeElement::QuoteBlock(children),
      ParseFrame::TabView(tabs) => TreeElement::TabView(tabs),
      ParseFrame::HtmlElement { tag, property } => TreeElement::HtmlElement { tag, property, children },
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseFrameKind {
  Bold,
  Italics,
  Underline,
  Strikethrough,
  Monospaced,
  Superscript,
  Subscript,
  Colored,
  Size,
  Collapsible,
  Footnote,
  QuoteBlock,
  TabView,
  HtmlElement
}

impl From<&ParseFrame> for ParseFrameKind {
  fn from(fr: &ParseFrame) -> ParseFrameKind {
    match fr {
      ParseFrame::Bold => Self::Bold,
      ParseFrame::Italics => Self::Italics,
      ParseFrame::Underline => Self::Underline,
      ParseFrame::Strikethrough => Self::Strikethrough,
      ParseFrame::Monospaced => Self::Monospaced,
      ParseFrame::Superscript => Self::Superscript,
      ParseFrame::Subscript => Self::Subscript,
      ParseFrame::Colored{..} => Self::Colored,
      ParseFrame::Size{..} => Self::Size,
      ParseFrame::Collapsible => Self::Collapsible,
      ParseFrame::Footnote{..} => Self::Footnote,
      ParseFrame::QuoteBlock => Self::QuoteBlock,
      ParseFrame::TabView{..} => Self::TabView,
      ParseFrame::HtmlElement{..} => Self::HtmlElement,
    }
  }
}

/// Convert ParseFrameKind into ParseFrame if it is possible to fill out every field of the ParseFrame.
impl From<&ParseFrameKind> for Option<ParseFrame> {
  fn from(kind: &ParseFrameKind) -> Option<ParseFrame> {
    match kind {
      ParseFrameKind::Bold => Some(ParseFrame::Bold),
      ParseFrameKind::Italics => Some(ParseFrame::Italics),
      ParseFrameKind::Underline => Some(ParseFrame::Underline),
      ParseFrameKind::Strikethrough => Some(ParseFrame::Strikethrough),
      ParseFrameKind::Monospaced => Some(ParseFrame::Monospaced),
      ParseFrameKind::Superscript => Some(ParseFrame::Superscript),
      ParseFrameKind::Subscript => Some(ParseFrame::Subscript),
      ParseFrameKind::Colored => None,
      ParseFrameKind::Size => None,
      ParseFrameKind::Collapsible => Some(ParseFrame::Collapsible),
      ParseFrameKind::Footnote => None,
      ParseFrameKind::QuoteBlock => Some(ParseFrame::QuoteBlock),
      ParseFrameKind::TabView => None,
      ParseFrameKind::HtmlElement => None,
    }
  }
}