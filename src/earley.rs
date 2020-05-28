use std::fmt;
use std::rc::Rc;

use crate::rules::{Production, Rule};
use crate::grammar::Grammar;

#[derive(Debug, Clone, PartialEq)]
pub struct LR0 {
  pub rule: Rc<Rule>,
  pub pos: usize,
}

impl LR0 {
  pub fn new(rule: &Rc<Rule>) -> Self {
    Self { rule: rule.clone(), pos: 0 }
  }

  pub fn is_active(&self) -> bool {
    self.pos < self.rule.len()
  }

  pub fn advance(&self) -> Self {
    assert!(self.is_active());
    Self {
      rule: self.rule.clone(),
      pos: self.pos + 1,
    }
  }

  pub fn next_production(&self) -> Option<&Production> {
    self.rule.productions.get(self.pos)
  }
}

impl fmt::Display for LR0 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} →", self.rule.symbol)?;
    for idx in 0..self.rule.len() {
      if idx == self.pos {
        write!(f, " ・")?;
      }
      write!(f, " {}", self.rule.productions[idx])?;
    }
    if !self.is_active() {
      write!(f, " ・")?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
  pub lr0: LR0,
  pub origin: usize,
}

impl State {
  pub fn new(lr0: LR0, origin: usize) -> Self {
    Self { lr0, origin }
  }

  pub fn advance(&self) -> Self {
    Self::new(self.lr0.advance(), self.origin)
  }
}

#[derive(Debug)]
pub struct Chart(Vec<Vec<State>>);

impl Chart {
  pub fn new(length: usize) -> Self {
    Self(vec![Vec::new(); length])
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn len_at(&self, k: usize) -> usize {
    self.0[k].len()
  }

  pub fn has(&self, k: usize, state: &State) -> bool {
    self.0[k].contains(state)
  }

  pub fn add(&mut self, k: usize, state: State) {
    if !self.has(k, &state) {
      self.0[k].push(state);
    }
  }

  /// Get an owned state so that passing around &mut chart is more ergonomic
  /// The clone is fairly cheap, only an rc + 2 usize, State would be copy if not
  /// for the Rc<Rule>
  fn get_state(&self, k: usize, idx: usize) -> State {
    self.0[k][idx].clone()
  }
}

impl IntoIterator for Chart {
  type Item = (usize, Vec<State>);
  type IntoIter = std::iter::Enumerate<std::vec::IntoIter<Vec<State>>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter().enumerate()
  }
}

impl fmt::Display for Chart {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for k in 0..self.len() {
      writeln!(f, "State {}:", k)?;
      for state in self.0[k].iter() {
        writeln!(f, "  {}..{}: {}", state.origin, k, state.lr0)?;
      }
    }
    Ok(())
  }
}

pub fn parse_chart(g: &Grammar, input: &[&str]) -> Chart {
  let mut chart = Chart::new(input.len() + 1);

  for rule in g.rules.get(&g.start).expect("grammar missing start rules") {
    chart.add(0, State::new(LR0::new(&rule), 0));
  }

  for k in 0..chart.len() {
    // need to use while loop because the number of states at k can expand during the loop
    let mut idx = 0;
    while idx < chart.len_at(k) {
      let state = chart.get_state(k, idx);
      idx += 1;

      match state.lr0.next_production() {
        None => completer(&mut chart, k, &state),
        Some(Production::Nonterminal(_)) => predictor(g, &mut chart, k, &state),
        Some(Production::Terminal(_)) => scanner(&mut chart, k, &state, input),
      };
    }
  }

  chart
}

fn completer(chart: &mut Chart, k: usize, state: &State) {
  assert!(!state.lr0.is_active(), "tried to complete active state");

  // lr0 has been completed, now look for states in the chart that are waiting for its symbol
  for idx in 0..chart.len_at(state.origin) {
    let other = chart.get_state(state.origin, idx);

    if let Some(np) = other.lr0.next_production() {
      if np.symbol_str() == state.lr0.rule.symbol_str() {
        // found one, advance its dot and add the new state to the chart *at k*,
        // because it's now waiting on a token there
        chart.add(k, other.advance())
      }
    }
  }
}

fn predictor(g: &Grammar, chart: &mut Chart, k: usize, state: &State) {
  assert!(state.lr0.is_active(), "tried to predict non-active state");
  assert!(
    state.lr0.next_production().unwrap().is_nonterminal(),
    "tried to predict a terminal"
  );

  // this lr0 is waiting for the next production
  // let's hypothesize that one of the rules that can build this production will
  // succeed at its current position
  let needed_symbol = state.lr0.next_production().unwrap().symbol_str();
  for wanted_rule in g
    .rules
    .get(needed_symbol)
    .unwrap_or_else(|| panic!("missing rules for production {}", needed_symbol))
  {
    chart.add(k, State::new(LR0::new(wanted_rule), k));

    if g.is_nullable(needed_symbol) {
      // automatically complete `state` early, because we know
      // it will be completable anyways, because its next_production may be produced
      // by empty input. If we don't do this, nullable rules won't be completed
      // correctly, because complete() won't run after predict() without a new symbol.
      chart.add(k, state.advance());
    }
  }
}

fn scanner(chart: &mut Chart, k: usize, state: &State, input: &[&str]) {
  assert!(state.lr0.is_active(), "tried to scan non-active state");
  assert!(
    state.lr0.next_production().unwrap().is_terminal(),
    "tried to scan a nonterminal"
  );

  let needed_symbol = state.lr0.next_production().unwrap().symbol_str();
  if k < input.len() && input[k] == needed_symbol {
    // advance the state to consume this token, and add to state k + 1, where
    // it will look for the next token
    chart.add(k + 1, state.advance());
  }
}

