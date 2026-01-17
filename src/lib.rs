mod tokenizer;
mod block;
mod inline;
mod ast;

fn parse(s: String) -> Result<(), Box<dyn std::error::Error>> {
  // get LF string
  let s = s.replace("\r\n", "\n"); // CRLF -> LF
  let s = s.replace("\r", "\n"); // CR -> LF

  let token = tokenizer::tokenize(s);
  let block_tree = block::parse(token);
  let ast = inline::parse(block_tree);
  

  Ok(())
}