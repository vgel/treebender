extern crate earley_rust;

use earley_rust::rules::Grammar;
use earley_rust::Err;

fn main() -> Result<(), Err> {
  let g: Grammar = Grammar::read_from_file("src/grammar.fgr")?;

  let sentence = "He said that Sue kissed Mary".to_ascii_lowercase();
  let sentence = sentence.split(' ').collect::<Vec<_>>();

  let trees = g.parse(&sentence);

  println!(
    "Parsed {} tree{}",
    trees.len(),
    if trees.len() == 1 { "" } else { "s" }
  );

  for (t, fs) in trees {
    println!("{}", t);
    println!("{}", fs);
    println!();
  }

  Ok(())
}
