use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::sync::{Arc, RwLock};

use crate::utils::Err;

/// Unpacked representation of a feature, that NodeRef::new_from_paths can turn into a Node
#[derive(Debug)]
pub struct Feature {
  /// Dotted path where each segment will be a node: "a.b.c" -> [a: [b: [c: ...]]]
  pub path: String,
  /// Unique string that will link features into a reentrant node, or None
  pub tag: Option<String>,
  /// What will end up at `path`. Will be unified with any other feature values with the same tag.
  pub value: NodeRef,
}

/// A raw node. Shouldn't be used, should always be wrapped in a NodeRef.
#[derive(Debug)]
pub(crate) enum Node {
  /// Top can unify with anything
  Top,
  /// A string-valued feature, such as "nom" in [case: nom]. Unifies with eq. Str nodes
  Str(String),
  /// An arc-containing node with arcs to other NodeRefs
  Edged(HashMap<String, NodeRef>),
  /// A node that has been forwarded to another node through unification.
  /// Before using a node, it should be dereferenced with Node::dereference to resolve its forward
  Forwarded(NodeRef),
}

impl Node {
  fn new_str(s: String) -> Self {
    Self::Str(s)
  }

  fn new_edged() -> Self {
    Self::Edged(HashMap::new())
  }

  fn is_top(&self) -> bool {
    matches!(self, Self::Top)
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

  fn edged(&self) -> Option<&HashMap<String, NodeRef>> {
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
        NodeRef::unify(existing, target)
      } else {
        arcs.insert(label, target);
        Ok(())
      }
    } else {
      Err(format!("unification failure: {}", label).into())
    }
  }
}

/// An interior-ly mutable ref to a Node.
#[derive(Debug)]
pub struct NodeRef(Arc<RwLock<Node>>);

impl NodeRef {
  pub fn new_top() -> Self {
    Node::Top.into()
  }

  pub fn new_str(s: String) -> Self {
    Node::new_str(s).into()
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
    Ok(n.into())
  }

  // List of (name, value, tag) triples
  pub fn new_from_paths<I>(paths: I) -> Result<NodeRef, Err>
  where
    I: IntoIterator<Item = Feature>,
  {
    let this: NodeRef = Node::new_edged().into();

    let mut tags: HashMap<String, NodeRef> = HashMap::new();
    for Feature { value, tag, path } in paths {
      if let Some(tag) = tag {
        if tags.contains_key(&tag) {
          let tagged = tags.get(&tag).unwrap();
          NodeRef::unify(value.clone(), tagged.clone())?;
        } else {
          tags.insert(tag.to_string(), value.clone());
        }
      }

      let mut current = this.clone();
      let mut parts = path.split('.').peekable();
      loop {
        let next = parts.next().expect("shouldn't be empty b/c path.len() > 0");
        let is_last = parts.peek().is_none();

        if is_last {
          current
            .borrow_mut()
            .push_edge(next.to_string(), value.clone())?;
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

  pub fn deep_clone(&self) -> NodeRef {
    let mut map = HashMap::new();
    self._deep_clone(&mut map);
    map.get(self).unwrap().clone()
  }

  pub fn dereference(self: NodeRef) -> NodeRef {
    if let Node::Forwarded(r) = &*self.borrow() {
      return Self::dereference(r.clone());
    }
    self
  }

  /// Unify two feature structures. Both will be mutated. Use deep_clone() if one needs to be preserved.
  pub fn unify(n1: NodeRef, n2: NodeRef) -> Result<(), Err> {
    let n1 = n1.dereference();
    let n2 = n2.dereference();

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

impl NodeRef {
  pub(crate) fn new(n: Node) -> Self {
    Self(Arc::new(RwLock::new(n)))
  }

  pub(crate) fn borrow(&self) -> RwLockReadGuard<Node> {
    self.0.read().expect("NodeRef lock poisoned!")
  }

  fn borrow_mut(&self) -> RwLockWriteGuard<Node> {
    self.0.write().expect("NodeRef lock poisoned!")
  }

  fn replace(&self, n: Node) -> Node {
    let mut write = self.borrow_mut();
    std::mem::replace(&mut *write, n)
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
}

impl Clone for NodeRef {
  /// Clones the ***rc*** of this NodeRef. Use deep_clone to clone the actual feature structure.
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl PartialEq for NodeRef {
  /// Compares NodeRefs via pointer equality. Does not dereference forwarding chains.
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for NodeRef {}

impl Hash for NodeRef {
  /// Hashes NodeRefs via pointer equality. Does not dereference forwarding chains.
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    let ptr = Arc::as_ptr(&self.0);
    ptr.hash(hasher)
  }
}

impl From<Node> for NodeRef {
  fn from(node: Node) -> Self {
    Self::new(node)
  }
}

// for fmt::Display impl
fn count_in_pointers(nref: NodeRef, seen: &mut HashMap<NodeRef, usize>) {
  let nref = nref.dereference();
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

// for fmt::Display impl
fn format_noderef(
  self_: NodeRef,
  counts: &HashMap<NodeRef, usize>,
  has_printed: &mut HashMap<NodeRef, usize>,
  indent: usize,
  f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
  let self_ = self_.dereference();

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
        format_noderef(value.clone(), counts, has_printed, 0, f)?;
        write!(f, " ]")
      } else {
        writeln!(f, "[")?;
        for (label, value) in arcs.iter() {
          write!(f, "{:indent$}{}: ", "", label, indent = indent + 2)?;
          format_noderef(value.clone(), counts, has_printed, indent + 2, f)?;
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
    format_noderef(self.clone(), &counts, &mut has_printed, 0, f)
  }
}
