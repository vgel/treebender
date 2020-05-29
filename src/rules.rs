use std::fmt;

use crate::featurestructure::NodeRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
  pub name: String,
}

impl Symbol {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone, PartialEq)]
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
  pub features: NodeRef,
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
    write!(f, "{}{} ->", self.symbol, self.features)?;
    for p in self.productions.iter() {
      write!(f, " {}", p)?;
    }
    Ok(())
  }
}
