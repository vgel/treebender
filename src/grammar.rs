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
    let (rules, nonterminals) = parse(s)?;
    if rules.is_empty() {
      return Err("empty ruleset".into());
    }

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
