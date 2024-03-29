use std::fmt;
use std::sync::Arc;

use crate::earley::Chart;
use crate::rules::{Grammar, Rule};
use crate::syntree::{Constituent, SynTree, Word};
use crate::utils::combinations;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForestState {
  rule: Arc<Rule>,
  span: (usize, usize),
}

impl ForestState {
  pub fn new(rule: &Arc<Rule>, start: usize, end: usize) -> Self {
    Self {
      rule: rule.clone(),
      span: (start, end),
    }
  }
}

impl fmt::Display for ForestState {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}: {}", self.span.0, self.span.1, self.rule)
  }
}

impl From<&ForestState> for Constituent<Arc<Rule>> {
  fn from(fs: &ForestState) -> Self {
    Self {
      value: fs.rule.clone(),
      span: fs.span,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forest(Vec<Vec<ForestState>>);

impl Forest {
  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Checks if a subtree has already been completed by make_trees(),
  /// or if it is a leaf and doesn't need to be completed
  fn subtree_is_complete(node: &SynTree<Arc<Rule>, String>) -> bool {
    if let Some((cons, children)) = node.get_branch() {
      cons.value.productions.len() == children.len()
    } else {
      // is a leaf
      true
    }
  }

  /// Takes a rule and search span, and returns a vec of all possible sequences
  /// of trees that correspond to the rule's productions.
  /// So for the situation:
  /// ```text
  /// g := '''
  ///   S -> x
  ///   S -> S S
  /// '''
  /// chart := parse(g, "x x x")
  /// chart.extend_out(g, S -> S S, start = 0, end = 3)
  /// ```
  /// , which, recall, has a chart that looks like:
  ///
  /// ```text
  /// 0..1: S -> x
  /// 0..2: S -> S S
  /// 0..3: S -> S S
  /// 1..2: S -> x
  /// 1..3: S -> S S
  /// 2..3: S -> x
  /// ```
  ///
  /// You'd get
  ///
  /// ```text
  /// [[(S -> x, 0..1), (S -> S S, (), 1..3)],
  ///  [(S -> S S, (), 0..2), (S -> x, 2..3)]]
  /// ```
  fn extend_out(
    &self,
    rule: &Rule,
    prod_idx: usize,
    search_start: usize,
    search_end: usize,
  ) -> Vec<Vec<SynTree<Arc<Rule>, String>>> {
    if prod_idx == rule.len() && search_start == search_end {
      // base case, we consumed the whole rule and the whole span together.
      // provide a single empty sequence as a base for prepending onto as we unwind the stack
      return vec![Vec::new()];
    } else if prod_idx == rule.len() || search_start == search_end {
      // we either ran out of productions before consuming everything, or ran out of stuff to consume before
      // satisfying all the productions. bail with 0 possible sequences.
      return Vec::new();
    }

    let next_production = &rule.productions[prod_idx];
    if next_production.is_nonterminal() {
      let wanted_symbol = &next_production.symbol;
      // look for potential next states to produce this production at the search start
      self.0[search_start]
        .iter()
        // only consider states that are contained within the search range, and have our wanted symbol
        .filter(|s| s.span.1 <= search_end && wanted_symbol == &s.rule.symbol)
        .flat_map(|state| {
          // recursively find possible sequences that start directly after this state
          // TODO: this is probably easily amenable to some dynamic programming to reduce repeated work
          self
            .extend_out(rule, prod_idx + 1, state.span.1, search_end)
            .into_iter()
            // if there are any, prepend an uncompleted tree headed by this state onto the sequence and throw it on the pile
            .map(move |mut seq| {
              seq.insert(0, SynTree::Branch(state.into(), Vec::new()));
              seq
            })
        })
        .collect()
    } else {
      // similar to the nonterminal case, but we don't have to search for multiple potential states --
      // all terminals with the same symbol_str are identical.
      let leaf = SynTree::Leaf(Word {
        value: next_production.symbol.to_string(),
        span: (search_start, search_start + 1),
      });

      // recursively find possible sequences, like before
      self
        .extend_out(rule, prod_idx + 1, search_start + 1, search_end)
        .into_iter()
        .map(move |mut seq| {
          // prepend our new leaf to them
          seq.insert(0, leaf.clone());
          seq
        })
        .collect()
    }
  }

  /// Takes a possibly-uncompleted tree, and returns all possible trees it describes.
  /// An uncompleted tree is a non-nullable constituent with 0 children. It needs to be passed
  /// into extend_out, and then glued onto
  fn make_trees(&self, tree: SynTree<Arc<Rule>, String>) -> Vec<SynTree<Arc<Rule>, String>> {
    if Self::subtree_is_complete(&tree) {
      vec![tree]
    } else {
      let (cons, _) = tree.get_branch().unwrap();
      self
        .extend_out(&cons.value, 0, cons.span.0, cons.span.1)
        .into_iter()
        .flat_map(|children| {
          let child_sets = children
            .into_iter()
            .map(|child| self.make_trees(child))
            .collect::<Vec<_>>();
          combinations(&child_sets)
            .into_iter()
            .map(|set| SynTree::Branch(cons.clone(), set))
        })
        .collect::<Vec<_>>()
    }
  }

  pub fn trees(&self, g: &Grammar) -> Vec<SynTree<Arc<Rule>, String>> {
    if self.is_empty() {
      Vec::new()
    } else {
      // seed our search with all LR0s that started at position 0, span to
      // the end of the string, and are named by the grammar's start symbol
      let root_states = self.0[0]
        .iter()
        .filter(|state| state.span.1 == self.len() && state.rule.symbol == g.start)
        .map(|state| SynTree::Branch(state.into(), Vec::new()));
      // use make_trees to generate all possible filled-in trees from each seed tree
      root_states.fold(
        Vec::<SynTree<Arc<Rule>, String>>::new(),
        |mut prev, tree| {
          let mut trees = self.make_trees(tree);
          prev.append(&mut trees);
          prev
        },
      )
    }
  }
}

impl From<Chart> for Forest {
  fn from(chart: Chart) -> Self {
    // the new chart will be indexed by origin location, and no rule can have
    // its origin at the end of the string, so len is chart.len - 1
    let mut v = vec![Vec::new(); chart.len() - 1];

    for (k, states) in chart.into_iter() {
      for state in states {
        // exclude unfinished rules that can't contribute to a tree
        if !state.lr0.is_active() {
          v.get_mut(state.origin)
            .expect("origin > input len")
            .push(ForestState::new(&state.lr0.rule, state.origin, k));
        }
      }
    }

    Self(v)
  }
}

impl fmt::Display for Forest {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for k in 0..self.len() {
      writeln!(f, "Origin {}:", k)?;
      for fs in self.0[k].iter() {
        writeln!(f, "  {}", fs)?;
      }
    }

    Ok(())
  }
}

#[test]
fn test_parse_chart() {
  let g: Grammar = r#"
    S -> x
    S -> S S
  "#
  .parse()
  .unwrap();

  let get_rule_with_len = |len: usize| {
    g.rules
      .get("S")
      .unwrap()
      .iter()
      .find(|r| r.len() == len)
      .unwrap()
  };

  let rule1 = get_rule_with_len(1);
  let rule2 = get_rule_with_len(2);

  let forest: Forest = crate::earley::parse_chart(&g, &["x", "x", "x"]).into();

  assert_eq!(
    forest,
    Forest(vec![
      vec![
        ForestState::new(rule1, 0, 1),
        ForestState::new(rule2, 0, 2),
        ForestState::new(rule2, 0, 3),
      ],
      vec![ForestState::new(rule1, 1, 2), ForestState::new(rule2, 1, 3),],
      vec![ForestState::new(rule1, 2, 3)],
    ])
  );

  println!("{}", forest);
}

#[test]
fn test_tree_generation() {
  // test the tree ambiguity problem that naive earley forest processing has
  // correct algorithm finds 2 trees:
  //  (S (S x) (S (S x) (S x)))           -> [x][xx]
  //  (S (S (S x) (S x)) (S x))           -> [xx][x]
  // naive algorithm finds 2 addl. spurious trees:
  //  (S (S x) (S x))                     -> [x][x]
  //  (S (S (S x) (S x)) (S (S x) (S x))) -> [xx][xx]

  let g = r#"
      S -> x
      S -> S S
    "#
  .parse()
  .unwrap();

  let forest: Forest = crate::earley::parse_chart(&g, &["x", "x", "x"]).into();
  let trees = forest.trees(&g);

  for tree in trees.iter() {
    println!("{}\n", tree);
  }

  assert_eq!(trees.len(), 2);
}
