#![allow(dead_code)]

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::parse_grammar::{ParsedFeature, ParsedFeatureValue};
use crate::Err;

#[derive(Debug)]
pub enum Node {
  Top,
  Str(String),
  Edged(HashMap<String, NodeRef>),
  Forwarded(NodeRef),
}

#[derive(Debug)]
pub struct NodeRef(Rc<RefCell<Node>>);

impl NodeRef {
  fn new(n: Node) -> Self {
    Self(Rc::new(RefCell::new(n)))
  }

  pub fn new_top() -> Self {
    Self::new(Node::Top)
  }

  pub fn new_str(s: String) -> Self {
    Self::new(Node::new_str(s))
  }

  /// Creates a NodeRef from a list of (name, noderef) features. Names CANNOT be dotted!
  pub fn new_with_edges<I>(edges: I) -> Result<Self, Err>
  where
    I: IntoIterator<Item = (String, NodeRef)>,
  {
    let mut n = Node::new_edged();
    for (label, target) in edges {
      assert!(
        !label.contains('.'),
        "new_with_edges cannot take dotted paths!"
      );

      n.push_edge(label, target)?;
    }
    Ok(Self::from(n))
  }

  // List of (name, value, tag) triples
  pub fn new_from_paths<I>(paths: I) -> Result<NodeRef, Err>
  where
    I: IntoIterator<Item = ParsedFeature>,
  {
    let this: NodeRef = Node::new_edged().into();

    let mut tags: HashMap<String, NodeRef> = HashMap::new();
    for feature in paths {
      let target = NodeRef::from(feature.value);

      if let Some(tag) = feature.tag {
        if tags.contains_key(&tag) {
          let tagged = tags.get(&tag).unwrap();
          Node::unify(target.clone(), tagged.clone())?;
        } else {
          tags.insert(tag.to_string(), target.clone());
        }
      }

      let mut current = this.clone();
      let mut parts = feature.dotted.split('.').peekable();
      loop {
        let next = parts.next().expect("shouldn't be empty b/c path.len() > 0");
        let is_last = parts.peek().is_none();

        if is_last {
          current
            .borrow_mut()
            .push_edge(next.to_string(), target.clone())?;
          break;
        } else {
          let new: NodeRef = Node::new_edged().into();
          current
            .borrow_mut()
            .push_edge(next.to_string(), new.clone())?;
          current = new;
        }
      }
    }

    Ok(this)
  }

  pub fn borrow(&self) -> Ref<Node> {
    self.0.borrow()
  }

  pub fn borrow_mut(&self) -> RefMut<Node> {
    self.0.borrow_mut()
  }

  pub fn replace(&self, n: Node) -> Node {
    self.0.replace(n)
  }

  fn _deep_clone(&self, seen: &mut HashMap<NodeRef, NodeRef>) -> NodeRef {
    if seen.contains_key(self) {
      return seen.get(self).unwrap().clone();
    }

    let n = self.borrow();
    let cloned = match &*n {
      Node::Forwarded(n1) => {
        let n1 = n1._deep_clone(seen);
        Self::new(Node::Forwarded(n1))
      }
      Node::Top => Self::new_top(),
      Node::Str(s) => Self::new_str(s.to_string()),
      Node::Edged(edges) => Self::new(Node::Edged(
        edges
          .iter()
          .map(|(k, v)| (k.clone(), v._deep_clone(seen)))
          .collect(),
      )),
    };
    seen.insert(self.clone(), cloned.clone());
    cloned
  }

  pub fn deep_clone(&self) -> NodeRef {
    let mut map = HashMap::new();
    self._deep_clone(&mut map);
    map.get(self).unwrap().clone()
  }
}

impl Clone for NodeRef {
  /// Clones the ***rc*** of this NodeRef. Use deep_clone to clone the actual feature structure.
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

// pointer equality
impl PartialEq for NodeRef {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for NodeRef {}

impl Hash for NodeRef {
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    self.0.as_ptr().hash(hasher)
  }
}

impl From<Node> for NodeRef {
  fn from(node: Node) -> Self {
    Self::new(node)
  }
}

impl From<ParsedFeatureValue> for NodeRef {
  fn from(v: ParsedFeatureValue) -> Self {
    match v {
      ParsedFeatureValue::Top => NodeRef::new_top(),
      ParsedFeatureValue::Str(s) => NodeRef::new_str(s),
    }
  }
}

impl Node {
  fn new_top() -> Self {
    Self::Top
  }

  fn new_str(s: String) -> Self {
    Self::Str(s)
  }

  fn new_edged() -> Self {
    Self::Edged(HashMap::new())
  }

  pub fn dereference(self_: NodeRef) -> NodeRef {
    if let Node::Forwarded(r) = &*self_.borrow() {
      return Self::dereference(r.clone());
    }
    self_
  }

  fn is_top(&self) -> bool {
    match self {
      Self::Top => true,
      _ => false,
    }
  }

  fn str(&self) -> Option<&str> {
    match self {
      Self::Str(s) => Some(s),
      _ => None,
    }
  }

  fn is_str(&self) -> bool {
    self.str().is_some()
  }

  pub fn edged(&self) -> Option<&HashMap<String, NodeRef>> {
    match self {
      Self::Edged(v) => Some(v),
      _ => None,
    }
  }

  fn edged_mut(&mut self) -> Option<&mut HashMap<String, NodeRef>> {
    match self {
      Self::Edged(v) => Some(v),
      _ => None,
    }
  }

  fn is_edged(&self) -> bool {
    self.edged().is_some()
  }

  #[allow(clippy::map_entry)]
  fn push_edge(&mut self, label: String, target: NodeRef) -> Result<(), Err> {
    if self.is_top() {
      *self = Self::new_edged();
    }

    if let Some(arcs) = self.edged_mut() {
      if arcs.contains_key(&label) {
        let existing = arcs[&label].clone();
        Self::unify(existing, target)
      } else {
        arcs.insert(label, target);
        Ok(())
      }
    } else {
      Err(format!("unification failure: {}", label).into())
    }
  }

  /// destroys both n1 and n2
  pub fn unify(n1: NodeRef, n2: NodeRef) -> Result<(), Err> {
    let n1 = Node::dereference(n1);
    let n2 = Node::dereference(n2);

    // quick check reference equality if the og nodes were forwarded to each other
    if n1 == n2 {
      return Ok(());
    }

    // if either is top forward to the other one w/o checking
    if n1.borrow().is_top() {
      n1.replace(Node::Forwarded(n2));
      return Ok(());
    } else if n2.borrow().is_top() {
      n2.replace(Node::Forwarded(n1));
      return Ok(());
    }

    // try to unify string values
    if n1.borrow().is_str() && n2.borrow().is_str() {
      let strs_equal = {
        let n1 = n1.borrow();
        let n2 = n2.borrow();
        n1.str().unwrap() == n2.str().unwrap()
      };
      if strs_equal {
        n1.replace(Node::Forwarded(n2));
        return Ok(());
      } else {
        return Err(
          format!(
            "unification failure: {} & {}",
            n1.borrow().str().unwrap(),
            n2.borrow().str().unwrap()
          )
          .into(),
        );
      }
    }

    if n1.borrow().is_edged() && n2.borrow().is_edged() {
      let n1 = n1.replace(Node::Forwarded(n2.clone()));
      let n2 = &mut *n2.borrow_mut();

      let n1arcs = n1.edged().unwrap();
      let n2arcs = n2.edged_mut().unwrap();

      for (label, value) in n1arcs.iter() {
        if n2arcs.contains_key(label) {
          // shared arc
          let other = n2arcs.get(label).unwrap();
          Self::unify(value.clone(), other.clone())?;
        } else {
          // complement arc
          n2arcs.insert(label.clone(), value.clone());
        }
      }

      return Ok(());
    }

    Err(format!("unification failure: {:#?} & {:#?}", n1, n2).into())
  }
}

fn count_in_pointers(nref: NodeRef, seen: &mut HashMap<NodeRef, usize>) {
  let nref = Node::dereference(nref);
  if seen.contains_key(&nref) {
    seen.entry(nref).and_modify(|cnt| *cnt += 1);
  } else {
    seen.insert(nref.clone(), 1);
    if let Some(arcs) = nref.borrow().edged() {
      for value in arcs.values() {
        count_in_pointers(value.clone(), seen);
      }
    }
  }
}

fn _fmt_noderef(
  self_: NodeRef,
  counts: &HashMap<NodeRef, usize>,
  has_printed: &mut HashMap<NodeRef, usize>,
  indent: usize,
  f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
  let self_ = Node::dereference(self_);

  if counts[&self_] > 1 && has_printed.contains_key(&self_) {
    return write!(f, "#{}", has_printed[&self_]);
  }

  if counts[&self_] > 1 {
    let id = has_printed.len();
    has_printed.insert(self_.clone(), id);
    write!(f, "#{} ", id)?;
  }

  let r = &*self_.borrow();
  match r {
    Node::Top => write!(f, "**top**"),
    Node::Str(s) => write!(f, "{}", s),
    Node::Edged(arcs) => {
      if arcs.is_empty() {
        write!(f, "[]")
      } else if arcs.len() == 1 {
        let (label, value) = arcs.iter().next().unwrap();
        write!(f, "[ {}: ", label)?;
        _fmt_noderef(value.clone(), counts, has_printed, 0, f)?;
        write!(f, " ]")
      } else {
        writeln!(f, "[")?;
        for (label, value) in arcs.iter() {
          write!(f, "{:indent$}{}: ", "", label, indent = indent + 2)?;
          _fmt_noderef(value.clone(), counts, has_printed, indent + 2, f)?;
          writeln!(f)?;
        }
        write!(f, "{:indent$}]", "", indent = indent)
      }
    }
    Node::Forwarded(_) => panic!("unexpected forward"),
  }
}

impl fmt::Display for NodeRef {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut counts = HashMap::new();
    count_in_pointers(self.clone(), &mut counts);
    let mut has_printed = HashMap::new();
    _fmt_noderef(self.clone(), &counts, &mut has_printed, 0, f)
  }
}

#[test]
fn test_construct_fs() {
  let root = NodeRef::new_from_paths(vec![
    ParsedFeature {
      dotted: "a.b".to_string(),
      tag: Some("1".to_string()),
      value: ParsedFeatureValue::Top,
    },
    ParsedFeature {
      dotted: "a.b.c".to_string(),
      tag: None,
      value: ParsedFeatureValue::Str("foo".to_string()),
    },
    ParsedFeature {
      dotted: "a.b.d".to_string(),
      tag: None,
      value: ParsedFeatureValue::Str("bar".to_string()),
    },
    ParsedFeature {
      dotted: "e".to_string(),
      tag: Some("1".to_string()),
      value: ParsedFeatureValue::Top,
    },
  ])
  .unwrap();

  println!("{}", root);
}

#[test]
fn test_unify_tags() {
  let fs1 = NodeRef::new_from_paths(vec![
    ParsedFeature {
      dotted: "a.b".to_string(),
      tag: Some("1".to_string()),
      value: ParsedFeatureValue::Top,
    },
    ParsedFeature {
      dotted: "c".to_string(),
      tag: Some("1".to_string()),
      value: ParsedFeatureValue::Top,
    },
  ])
  .unwrap();

  let fs2 = NodeRef::new_from_paths(vec![ParsedFeature {
    dotted: "c".to_string(),
    tag: None,
    value: ParsedFeatureValue::Str("foo".to_string()),
  }])
  .unwrap();

  Node::unify(fs1.clone(), fs2).unwrap();

  println!("{}", fs1);
}