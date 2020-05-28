use std::fmt;

use crate::featurestructure::NodeRef;

#[derive(Debug, PartialEq)]
pub struct Symbol {
  pub name: String,
  pub features: NodeRef,
}

impl Symbol {
  pub fn new(name: String, features: NodeRef) -> Self {
    Self {
      name,
      features,
    }
  }
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}{}", self.name, self.features)
  }
}

#[derive(Debug, PartialEq)]
pub enum Production {
  Terminal(String),
  Nonterminal(Symbol),
}

impl Production {
  pub fn symbol_str(&self) -> &str {
    match self {
      Self::Terminal(s) => s,
      Self::Nonterminal(s) => &s.name,
    }
  }

  pub fn is_terminal(&self) -> bool {
    match self {
      Self::Terminal(_) => true,
      _ => false,
    }
  }

  pub fn is_nonterminal(&self) -> bool {
    match self {
      Self::Nonterminal(_) => true,
      _ => false,
    }
  }
}

impl fmt::Display for Production {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Terminal(s) => write!(f, "{}", s),
      Self::Nonterminal(s) => write!(f, "{}", s),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Rule {
  pub symbol: Symbol,
  pub productions: Vec<Production>,
}

impl Rule {
  pub fn len(&self) -> usize {
    self.productions.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn symbol_str(&self) -> String {
    self.symbol.name.clone()
  }
}

impl std::fmt::Display for Rule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ->", self.symbol)?;
    for p in self.productions.iter() {
      write!(f, " {}", p)?;
    }
    Ok(())
  }
}

// #[test]
// fn test_parse_grammar() {
//   let g = Grammar::parse(
//     "S".to_string(),
//     r#"
//       S -> N[ case: nom; num: #1; ] IV[ num: #1; ]
//       S -> N[ case: nom; pron: #1; num: #2; ] TV[ num: #2; ] N[ case: acc; needs_pron: #1; ]
//       S -> N[ case: nom; num: #1; ] CV[ num: #num; ] Comp S

//       N[ num: sg; pron: she; ]     -> Mary
//       IV[ num: top; tense: past; ] -> fell
//       TV[ num: top; tense: past; ] -> kissed
//       CV[ num: top; tense: past; ] -> said
//       Comp -> that
//     "#,
//   )
//   .unwrap();

//   let nt: HashSet<String> = ["S", "N", "IV", "TV", "CV", "Comp"]
//     .iter()
//     .map(|&s| s.to_string())
//     .collect();
//   assert_eq!(g.nonterminals, nt);
// }

// #[test]
// fn test_find_nullables() {
//   let g = Grammar::parse(
//     "S".to_string(),
//     r#"
//       S -> A B
//       A -> c
//       B -> D D
//       D ->
//     "#,
//   )
//   .unwrap();

//   let nl: HashSet<String> = ["B", "D"].iter().map(|&s| s.to_string()).collect();
//   assert_eq!(g.nullables, nl);
// }

// #[test]
// fn test_grammar_display() {
//   let g = Grammar::parse(
//     "S".to_string(),
//     r#"
//       S -> N[ case: nom; num: #1; ] IV[ num: #1; ]
//       S -> N[ case: nom; pron: #1; num: #2; ] TV[ num: #2; ] N[ case: acc; needs_pron: #1; ]
//       S -> N[ case: nom; num: #1; ] CV[ num: #num; ] Comp S

//       N[ num: sg; pron: she; ]     -> Mary
//       IV[ num: top; tense: past; ] -> fell
//       TV[ num: top; tense: past; ] -> kissed
//       CV[ num: top; tense: past; ] -> said
//       Comp -> that
//     "#,
//   )
//   .unwrap();

//   println!("{}", g);
// }
