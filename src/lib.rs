mod tokenizer;
mod block;
mod inline;

fn parse(s: &str) -> Result<(), Box<dyn std::error::Error>> {
  let token = tokenizer::tokenize(s);
  Ok(())
}