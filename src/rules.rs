use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use crate::featurestructure::NodeRef;
use crate::utils::Err;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProductionKind {
  Terminal,
  Nonterminal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Production {
  pub kind: ProductionKind,
  pub symbol: String,
}

impl Production {
  pub fn new_terminal(symbol: String) -> Self {
    Self {
      kind: ProductionKind::Terminal,
      symbol,
    }
  }

  pub fn new_nonterminal(symbol: String) -> Self {
    Self {
      kind: ProductionKind::Nonterminal,
      symbol,
    }
  }

  pub fn is_terminal(&self) -> bool {
    self.kind == ProductionKind::Terminal
  }

  pub fn is_nonterminal(&self) -> bool {
    self.kind == ProductionKind::Nonterminal
  }
}

impl fmt::Display for Production {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.symbol)
  }
}

#[derive(Debug, PartialEq)]
pub struct Rule {
  pub symbol: String,
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

#[derive(Debug)]
pub struct Grammar {
  pub start: String,
  pub rules: HashMap<String, Vec<Rc<Rule>>>,
  nullables: HashSet<String>,
  nonterminals: HashSet<String>,
}

impl std::fmt::Display for Grammar {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "//** start: {}", self.start)?;
    write!(f, "//** nonterminals:")?;
    for nt in self.nonterminals.iter() {
      write!(f, " {}", nt)?;
    }
    writeln!(f)?;

    write!(f, "//** nullables:")?;
    for nt in self.nullables.iter() {
      write!(f, " {}", nt)?;
    }
    writeln!(f)?;

    for rule in self.rules.values().flatten() {
      writeln!(f, "{}\n", rule)?;
    }

    Ok(())
  }
}

impl Grammar {
  pub fn new(rules: Vec<Rule>) -> Result<Self, Err> {
    assert!(!rules.is_empty());

    let nonterminals: HashSet<String> = rules.iter().map(|r| r.symbol.clone()).collect();
    let start = rules[0].symbol.clone();

    for r in rules.iter() {
      for p in r.productions.iter() {
        if p.is_nonterminal() && !nonterminals.contains(&p.symbol) {
          return Err(format!("missing rules for nonterminal {}", p.symbol).into());
        }
      }
    }

    let rules: HashMap<String, Vec<Rc<Rule>>> =
      rules.into_iter().fold(HashMap::new(), |mut map, rule| {
        map
          .entry(rule.symbol.clone())
          .or_insert_with(Vec::new)
          .push(Rc::new(rule));
        map
      });

    let nullables = Self::find_nullables(&rules);

    Ok(Self {
      start,
      rules,
      nonterminals,
      nullables,
    })
  }

  pub fn is_nullable(&self, s: &str) -> bool {
    self.nullables.contains(s)
  }
}

impl Grammar {
  fn rule_is_nullable(nullables: &HashSet<String>, rule: &Rule) -> bool {
    rule.is_empty()
      || rule
        .productions
        .iter()
        .all(|p| p.is_nonterminal() && nullables.contains(&p.symbol))
  }

  fn find_nullables(rules: &HashMap<String, Vec<Rc<Rule>>>) -> HashSet<String> {
    let mut nullables: HashSet<String> = HashSet::new();

    let mut last_length = 1;
    while last_length != nullables.len() {
      last_length = nullables.len();
      for r in rules.values().flatten() {
        if !nullables.contains(&r.symbol) && Self::rule_is_nullable(&nullables, &r) {
          nullables.insert(r.symbol.clone());
        }
      }
    }

    nullables
  }
}

#[test]
fn test_parse_grammar() {
  let g: Grammar = r#"
       S -> N[ case: nom, num: #1 ] IV[ num: #1 ]
       S -> N[ case: nom, pron: #1, num: #2 ] TV[ num: #2 ] N[ case: acc, needs_pron: #1 ]
       S -> N[ case: nom, num: #1 ] CV[ num: #num ] Comp S

       N[ num: sg, pron: she ]     -> mary
       IV[ num: top, tense: past ] -> fell
       TV[ num: top, tense: past ] -> kissed
       CV[ num: top, tense: past ] -> said
       Comp -> that
     "#
  .parse()
  .unwrap();

  let nonterminals: HashSet<String> = ["S", "N", "IV", "TV", "CV", "Comp"]
    .iter()
    .map(|&s| s.to_string())
    .collect();
  assert_eq!(nonterminals, g.nonterminals);
  assert_eq!(g.rules.len(), 6);

  assert_eq!(g.rules.get("S").unwrap().len(), 3);
  assert_eq!(g.rules.get("N").unwrap().len(), 1);
  assert_eq!(g.rules.get("IV").unwrap().len(), 1);
  assert_eq!(g.rules.get("TV").unwrap().len(), 1);
  assert_eq!(g.rules.get("CV").unwrap().len(), 1);
  assert_eq!(g.rules.get("Comp").unwrap().len(), 1);
  assert!(g.rules.get("that").is_none());
  assert!(g.rules.get("mary").is_none());
}

#[test]
fn test_find_nullables() {
  let g: Grammar = r#"
      S -> A B
      A -> c
      B -> D D
      D ->
    "#
  .parse()
  .unwrap();

  let nl: HashSet<String> = ["B", "D"].iter().map(|&s| s.to_string()).collect();
  assert_eq!(g.nullables, nl);
}
