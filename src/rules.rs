use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

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
  pub fn new(rules: Vec<Rule>) -> Self {
    assert!(!rules.is_empty());

    let nonterminals: HashSet<String> = rules.iter().map(|r| r.symbol.name.clone()).collect();
    let start = rules[0].symbol.name.clone();

    let rules: HashMap<String, Vec<Rc<Rule>>> =
      rules.into_iter().fold(HashMap::new(), |mut map, rule| {
        map
          .entry(rule.symbol.name.clone())
          .or_insert_with(Vec::new)
          .push(Rc::new(rule));
        map
      });

    let nullables = Self::find_nullables(&rules);

    Self {
      start,
      rules,
      nonterminals,
      nullables,
    }
  }

  pub fn is_nullable(&self, s: &str) -> bool {
    self.nullables.contains(s)
  }
}

impl Grammar {
  fn rule_is_nullable(nullables: &HashSet<String>, rule: &Rule) -> bool {
    rule.is_empty()
      || rule.productions.iter().all(|p| match p {
        Production::Nonterminal(s) => nullables.contains(&s.name),
        Production::Terminal(s) => nullables.contains(s),
      })
  }

  fn find_nullables(rules: &HashMap<String, Vec<Rc<Rule>>>) -> HashSet<String> {
    let mut nullables: HashSet<String> = HashSet::new();

    let mut last_length = 1;
    while last_length != nullables.len() {
      last_length = nullables.len();
      for r in rules.values().flatten() {
        if !nullables.contains(&r.symbol.name) && Self::rule_is_nullable(&nullables, &r) {
          nullables.insert(r.symbol.name.clone());
        }
      }
    }

    nullables
  }
}

#[test]
fn test_parse_grammar() {
  let g: Grammar = r#"
       S -> N[ case: nom, num: #1 ] IV[ num: #1 ];
       S -> N[ case: nom, pron: #1, num: #2 ] TV[ num: #2 ] N[ case: acc, needs_pron: #1 ];
       S -> N[ case: nom, num: #1 ] CV[ num: #num ] Comp S;

       N[ num: sg, pron: she ]     -> Mary;
       IV[ num: top, tense: past ] -> fell;
       TV[ num: top, tense: past ] -> kissed;
       CV[ num: top, tense: past ] -> said;
       Comp -> that;
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
  assert!(g.rules.get("Mary").is_none());
}

#[test]
fn test_find_nullables() {
  let g: Grammar = r#"
      S -> A B;
      A -> c;
      B -> D D;
      D ->;
    "#
  .parse()
  .unwrap();

  let nl: HashSet<String> = ["B", "D"].iter().map(|&s| s.to_string()).collect();
  assert_eq!(g.nullables, nl);
}
