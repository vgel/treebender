#[macro_use]
extern crate lazy_static;

pub mod earley;
pub mod featurestructure;
pub mod forest;
pub mod parse_grammar;
pub mod rules;
pub mod syntree;

use std::rc::Rc;

use crate::earley::{parse_chart, Chart};
use crate::featurestructure::NodeRef;
use crate::forest::Forest;
use crate::rules::{Grammar, Rule};
use crate::syntree::SynTree;

pub type Err = Box<dyn std::error::Error + 'static>;

impl Grammar {
  fn parse_chart(&self, input: &[&str]) -> Chart {
    parse_chart(&self, input)
  }

  fn parse_forest(&self, input: &[&str]) -> Forest {
    Forest::from(self.parse_chart(input))
  }

  pub fn unify_tree(tree: SynTree<Rc<Rule>, String>) -> Result<NodeRef, Err> {
    match tree {
      SynTree::Leaf(_) => Ok(NodeRef::new_top()),
      SynTree::Branch(cons, children) => {
        let features = cons.value.features.deep_clone();

        for (idx, child) in children.into_iter().enumerate() {
          let child = Self::unify_tree(child)?;
          let to_unify = NodeRef::new_with_edges(vec![(format!("child-{}", idx), child)])?;
          NodeRef::unify(features.clone(), to_unify)?;
        }

        Ok(features)
      }
    }
  }

  pub fn parse(&self, input: &[&str]) -> Vec<NodeRef> {
    let forest = self.parse_forest(input);
    let trees = forest.trees(&self);
    trees
      .into_iter()
      .filter_map(|t| Self::unify_tree(t).map(Some).unwrap_or(None))
      .collect::<Vec<_>>()
  }
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
