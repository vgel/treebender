pub mod parse_grammar;

pub use parse_grammar::*;

#[cfg(test)]
mod tests {
  use crate::Grammar;

  macro_rules! example_file {
    ($filename:expr) => {
      (
        $filename,
        include_str!(concat!("../../examples/", $filename)),
      )
    };
  }

  #[test]
  fn smoke_test_examples() {
    let examples = [
      example_file!("asl-wordorder.fgr"),
      example_file!("dative-shift.fgr"),
      example_file!("no-features.fgr"),
      example_file!("reflexives.fgr"),
    ];

    for (filename, src) in examples {
      assert!(src.parse::<Grammar>().is_ok(), "failed to parse {filename}");
    }
  }
}
