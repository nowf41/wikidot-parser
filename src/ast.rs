#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSize(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url(pub String); // TODO validate

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
  pub struct BlockCell {
    pub val: Vec<crate::tokenizer::Token>,
    pub style: Option<Style>,
    pub spanning: std::num::NonZeroUsize,
  }

  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct Cell {
    pub val: Vec<crate::ast::TreeElement>,
    pub style: Option<Style>,
    pub spanning: std::num::NonZeroUsize,
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeElement {
  Paragraph(Vec<TreeElement>),
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
  Link{href: Url, open_in_new_tab: bool, name: String}, // TODO implement parsing name as wikidot string
  InternalLink{href: String, open_in_new_tab: bool, name: String}, // TODO implement parsing name as wikidot string
  Collapsible(Vec<TreeElement>),
  Footnote{id: u32, children: Vec<TreeElement>}, // idは構文解析時に自動的に生成
  QuoteBlock(Vec<TreeElement>),
  Iframe(String), // the value is raw HTML element string
  Tab{
    title: String,
    children: Vec<TreeElement>,
  },
  TabView(Vec<TreeElement>), // only holds Tabs
  Table(Vec<Vec<table_cell::Cell>>),
  NewLine,

  HtmlElement{tag: String, property: Vec<(String, String)>, children: Vec<TreeElement>},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFrame {
  Paragraph,
  Bold,
  Italics,
  Underline,
  Strikethrough,
  Monospaced,
  Superscript,
  Subscript,
  Colored{red: u8, green: u8, blue: u8},
  Size{scale: CssSize},
  // Link does not contain children
  // InternalLink does not contain children
  Collapsible,
  Footnote{id: u32},
  QuoteBlock,
  // Iframe is a single element. The values are written in HTML and they won't be parsed.
  Tab(String),
  TabView, // this is a div element internally, just for showing renderers begin of TabView
  // Table does not contain TreeElement children

  HtmlElement{tag: String, property: Vec<(String, String)>}, // should be filtered by its tag
}

impl ParseFrame {
  pub fn into_tree_element(self, children: Vec<TreeElement>) -> TreeElement {
    match self {
      ParseFrame::Paragraph => TreeElement::Paragraph(children),
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
      ParseFrame::Tab(title) => TreeElement::Tab{title, children},
      ParseFrame::TabView => TreeElement::TabView(children),
      ParseFrame::HtmlElement { tag, property } => TreeElement::HtmlElement { tag, property, children },
    }
  }

  pub fn get_kind(&self) -> ParseFrameKind {
    match self {
      ParseFrame::Paragraph => ParseFrameKind::Paragraph,
      ParseFrame::Bold => ParseFrameKind::Bold,
      ParseFrame::Italics => ParseFrameKind::Italics,
      ParseFrame::Underline => ParseFrameKind::Underline,
      ParseFrame::Strikethrough => ParseFrameKind::Strikethrough,
      ParseFrame::Monospaced => ParseFrameKind::Monospaced,
      ParseFrame::Superscript => ParseFrameKind::Superscript,
      ParseFrame::Subscript => ParseFrameKind::Subscript,
      ParseFrame::Colored{..} => ParseFrameKind::Colored,
      ParseFrame::Size{..} => ParseFrameKind::Size,
      ParseFrame::Collapsible => ParseFrameKind::Collapsible,
      ParseFrame::Footnote{..} => ParseFrameKind::Footnote,
      ParseFrame::QuoteBlock =>  ParseFrameKind::QuoteBlock,
      ParseFrame::Tab{..} => ParseFrameKind::Tab,
      ParseFrame::TabView => ParseFrameKind::TabView,
      ParseFrame::HtmlElement{tag, ..} => ParseFrameKind::HtmlElement{tag: tag.clone()},
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFrameKind {
  Paragraph,
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
  Tab,
  TabView,
  HtmlElement{tag: String},
}