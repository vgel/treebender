#![allow(unused)]

use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::str::FromStr;

use crate::earley::{parse_chart, Chart};
use crate::featurestructure::NodeRef;
use crate::forest::{unify_tree, Forest};
use crate::parse_grammar::parse;
use crate::rules::{Production, Rule, Symbol};
use crate::syntree::SynTree;
use crate::Err;

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

  pub fn is_nullable(&self, s: &str) -> bool {
    self.nullables.contains(s)
  }

  fn parse_chart(&self, input: &[&str]) -> Chart {
    parse_chart(&self, input)
  }

  fn parse_forest(&self, input: &[&str]) -> Forest {
    Forest::from(self.parse_chart(input))
  }

  pub fn parse(&self, input: &[&str]) -> Vec<NodeRef> {
    let forest = self.parse_forest(input);
    let trees = forest.trees(&self);
    trees
      .into_iter()
      .filter_map(|t| unify_tree(t).map(Some).unwrap_or(None))
      .collect::<Vec<_>>()
  }
}

impl FromStr for Grammar {
  type Err = Err;

  /// Parses a grammar from a string. Assumes the first rule's symbol
  /// is the start symbol.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let rules = parse(s)?;
    if rules.is_empty() {
      return Err("empty ruleset".into());
    }

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

    Ok(Self {
      start,
      rules,
      nonterminals,
      nullables,
    })
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

#[test]
fn test_unification_blocking() {
  let g: Grammar = r#"
    S -> N[ case: nom, pron: #1 ] TV N[ case: acc, needs_pron: #1 ];
    TV -> likes;
    N[ case: nom, pron: she ] -> she;
    N[ case: nom, pron: he ] -> he;
    N[ case: acc, pron: he ] -> him;
    N[ case: acc, pron: ref, needs_pron: he ] -> himself;
  "#
  .parse()
  .unwrap();

  assert_eq!(g.parse(&["he", "likes", "himself"]).len(), 1);
  assert_eq!(g.parse(&["he", "likes", "him"]).len(), 1);
  assert_eq!(g.parse(&["she", "likes", "him"]).len(), 1);

  assert_eq!(g.parse(&["himself", "likes", "himself"]).len(), 0);
  assert_eq!(g.parse(&["she", "likes", "himself"]).len(), 0);
  assert_eq!(g.parse(&["himself", "likes", "him"]).len(), 0);
}
