use std::collections::HashMap;
use std::fmt;

use crate::utils::Err;

/// Index type for the node arena
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeIdx(pub u32);

/// Unpacked representation of a feature, that NodeArena::new_from_paths can turn into a Node
#[derive(Debug)]
pub struct Feature {
  /// Dotted path where each segment will be a node: "a.b.c" -> [a: [b: [c: ...]]]
  pub path: String,
  /// Unique string that will link features into a reentrant node, or None
  pub tag: Option<String>,
  /// What will end up at `path`. Will be unified with any other feature values with the same tag.
  pub value: NodeIdx,
}

/// A node in the feature structure graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
  /// Top can unify with anything
  Top,
  /// A string-valued feature, such as "nom" in [case: nom]. Unifies with eq. Str nodes
  Str(String),
  /// An arc-containing node with arcs to other NodeIdxs
  Edged(HashMap<String, NodeIdx>),
  /// A node that has been forwarded to another node through unification.
  /// Before using a node, it should be dereferenced to resolve its forward
  Forwarded(NodeIdx),
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

  fn edged(&self) -> Option<&HashMap<String, NodeIdx>> {
    match self {
      Self::Edged(v) => Some(v),
      _ => None,
    }
  }

  fn edged_mut(&mut self) -> Option<&mut HashMap<String, NodeIdx>> {
    match self {
      Self::Edged(v) => Some(v),
      _ => None,
    }
  }

  fn is_edged(&self) -> bool {
    self.edged().is_some()
  }
}

/// An arena that stores all nodes and provides methods to operate on them
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeArena {
  nodes: Vec<Node>,
}

impl NodeArena {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn alloc(&mut self, node: Node) -> NodeIdx {
    let idx = self.nodes.len() as u32;
    self.nodes.push(node);
    NodeIdx(idx)
  }

  pub fn replace(&mut self, idx: NodeIdx, node: Node) -> Node {
    std::mem::replace(&mut self.nodes[idx.0 as usize], node)
  }

  pub fn alloc_top(&mut self) -> NodeIdx {
    self.alloc(Node::Top)
  }

  pub fn alloc_str(&mut self, s: String) -> NodeIdx {
    self.alloc(Node::new_str(s))
  }

  pub fn alloc_edged(&mut self) -> NodeIdx {
    self.alloc(Node::new_edged())
  }

  /// Display a NodeIdx
  pub fn display(&self, idx: NodeIdx) -> NodeDisplay {
    NodeDisplay { arena: self, idx }
  }

  /// Creates a Node from a list of (name, noderef) features. Names CANNOT be dotted!
  pub fn alloc_from_edges<I>(&mut self, edges: I) -> Result<NodeIdx, Err>
  where
    I: IntoIterator<Item = (String, NodeIdx)>,
  {
    let node = self.alloc_edged();

    for (label, target) in edges {
      assert!(
        !label.contains('.'),
        "new_with_edges cannot take dotted paths!"
      );

      self.push_edge(node, label, target)?; // error if unification failure
    }

    Ok(node)
  }

  pub fn alloc_from_features<I>(&mut self, paths: I) -> Result<NodeIdx, Err>
  where
    I: IntoIterator<Item = Feature>,
  {
    let root = self.alloc_edged();

    let mut tags: HashMap<String, NodeIdx> = HashMap::new();
    for Feature { value, tag, path } in paths {
      if let Some(tag) = tag {
        if tags.contains_key(&tag) {
          let tagged = tags[&tag];
          self.unify(value, tagged)?;
        } else {
          tags.insert(tag.to_string(), value);
        }
      }

      let mut current = root;
      let mut parts = path.split('.').peekable();
      loop {
        let next = parts.next().expect("shouldn't be empty b/c path.len() > 0");
        let is_last = parts.peek().is_none();

        if is_last {
          self.push_edge(current, next.to_string(), value)?;
          break;
        } else {
          let new = self.alloc_edged();
          self.push_edge(current, next.to_string(), new)?;
          current = new;
        }
      }
    }

    Ok(root)
  }

  /// Get an idx. Assumes valid, panics on OOB
  pub fn get(&self, idx: NodeIdx) -> &Node {
    self.nodes.get(idx.0 as usize).expect("Invalid NodeIdx")
  }

  /// Mutably get an idx. Assumes valid, panics on OOB
  pub fn get_mut(&mut self, idx: NodeIdx) -> &mut Node {
    self.nodes.get_mut(idx.0 as usize).expect("Invalid NodeIdx")
  }

  pub fn forward_to(&mut self, target: NodeIdx, to: NodeIdx) {
    self.nodes[target.0 as usize] = Node::Forwarded(to);
  }

  pub fn is_top(&self, n: NodeIdx) -> bool {
    self.get(n).is_top()
  }

  pub fn is_str(&self, n: NodeIdx) -> bool {
    self.get(n).is_str()
  }

  pub fn is_edged(&self, n: NodeIdx) -> bool {
    self.get(n).is_edged()
  }

  fn str(&self, n: NodeIdx) -> Option<&str> {
    self.get(n).str()
  }

  fn edged(&self, n: NodeIdx) -> Option<&HashMap<String, NodeIdx>> {
    self.get(n).edged()
  }

  fn edged_mut(&mut self, n: NodeIdx) -> Option<&mut HashMap<String, NodeIdx>> {
    self.get_mut(n).edged_mut()
  }

  #[allow(clippy::map_entry)]
  fn push_edge(&mut self, parent: NodeIdx, label: String, target: NodeIdx) -> Result<(), Err> {
    let node = self.get_mut(parent);

    if node.is_top() {
      *node = Node::new_edged();
    }

    if let Some(arcs) = node.edged_mut() {
      if arcs.contains_key(&label) {
        let existing = arcs[&label];
        self.unify(existing, target)?;
      } else {
        arcs.insert(label, target);
      }
      return Ok(());
    }

    Err(format!("unification failure: {}", label).into())
  }

  pub fn dereference(&self, mut idx: NodeIdx) -> NodeIdx {
    while let Node::Forwarded(r) = self.get(idx) {
      idx = *r;
    }
    idx
  }

  /// Unify two feature structures within this arena. Both may be mutated.
  pub fn unify(&mut self, n1: NodeIdx, n2: NodeIdx) -> Result<(), Err> {
    let n1 = self.dereference(n1);
    let n2 = self.dereference(n2);

    // if same node, already unified
    if n1 == n2 {
      return Ok(());
    }

    // If either is top, forward to the other
    if self.is_top(n1) {
      self.forward_to(n1, n2);
      return Ok(());
    } else if self.is_top(n2) {
      self.forward_to(n2, n1);
      return Ok(());
    }

    // try to unify string values
    if self.is_str(n1) && self.is_str(n2) {
      let n1_str = self.str(n1).unwrap();
      let n2_str = self.str(n2).unwrap();

      if n1_str == n2_str {
        self.forward_to(n1, n2);
        return Ok(());
      } else {
        return Err(format!("unification failure: {n1_str} & {n2_str}").into());
      }
    }

    // if both are edged, unify their contents
    if self.is_edged(n1) && self.is_edged(n2) {
      let n1 = self.replace(n1, Node::Forwarded(n2));
      let n1arcs = n1.edged().unwrap();

      for (label, value) in n1arcs.iter() {
        if self.edged(n2).unwrap().contains_key(label) {
          // shared arc
          let other = self.edged(n2).unwrap().get(label).unwrap();
          self.unify(*value, *other)?;
        } else {
          // complement arc
          self.edged_mut(n2).unwrap().insert(label.clone(), *value);
        }
      }

      return Ok(());
    }

    Err(
      format!(
        "unification failure: {:?} & {:?}",
        self.get(n1),
        self.get(n2)
      )
      .into(),
    )
  }
}

/// Helper struct for displaying a node
#[derive(Clone)]
pub struct NodeDisplay<'a> {
  pub arena: &'a NodeArena,
  pub idx: NodeIdx,
}

impl fmt::Display for NodeDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut counts = HashMap::new();
    count_in_pointers(self, &mut counts);
    let mut has_printed = HashMap::new();
    format_node(self, &counts, &mut has_printed, 0, f)
  }
}

// for fmt::Display impl
#[allow(clippy::map_entry)]
fn count_in_pointers(n: &NodeDisplay, seen: &mut HashMap<NodeIdx, usize>) {
  let nref = n.arena.dereference(n.idx);
  if seen.contains_key(&nref) {
    seen.entry(nref).and_modify(|cnt| *cnt += 1);
  } else {
    seen.insert(nref, 1);
    if let Some(arcs) = n.arena.edged(nref) {
      for value in arcs.values() {
        count_in_pointers(
          &NodeDisplay {
            arena: n.arena,
            idx: *value,
          },
          seen,
        );
      }
    }
  }
}

// for fmt::Display impl
fn format_node(
  nd: &NodeDisplay,
  counts: &HashMap<NodeIdx, usize>,
  has_printed: &mut HashMap<NodeIdx, usize>,
  indent: usize,
  f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
  let arena = nd.arena;
  let idx = arena.dereference(nd.idx);

  if counts[&idx] > 1 && has_printed.contains_key(&idx) {
    return write!(f, "#{}", has_printed[&idx]);
  }

  if counts[&idx] > 1 {
    let id = has_printed.len();
    has_printed.insert(idx, id);
    write!(f, "#{} ", id)?;
  }

  let r = nd.arena.get(idx);
  match r {
    Node::Top => write!(f, "**top**"),
    Node::Str(s) => write!(f, "{}", s),
    Node::Edged(arcs) => {
      if arcs.is_empty() {
        write!(f, "[]")
      } else if arcs.len() == 1 {
        let (label, value) = arcs.iter().next().unwrap();
        write!(f, "[ {}: ", label)?;
        format_node(
          &NodeDisplay { arena, idx: *value },
          counts,
          has_printed,
          0,
          f,
        )?;
        write!(f, " ]")
      } else {
        writeln!(f, "[")?;
        for (label, value) in arcs.iter() {
          write!(f, "{:indent$}{}: ", "", label, indent = indent + 2)?;
          format_node(
            &NodeDisplay { arena, idx: *value },
            counts,
            has_printed,
            indent + 2,
            f,
          )?;
          writeln!(f)?;
        }
        write!(f, "{:indent$}]", "", indent = indent)
      }
    }
    Node::Forwarded(_) => panic!("unexpected forward"),
  }
}
