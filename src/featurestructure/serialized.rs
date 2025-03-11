use std::collections::HashMap;

use super::node::{Node, NodeArena, NodeIdx};

/// A noderef that's been serialized into a tree structure. Nodes with multiple
/// in-pointers are duplicated.
/// IMPORTANT: **top** is /stripped out/. All top features will not be present in
/// the serialized tree.
#[derive(Debug, Clone)]
pub enum SerializedNode {
  Str(String),
  Edged(HashMap<String, SerializedNode>),
}

impl SerializedNode {
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Self::Str(s) => Some(s.as_str()),
      _ => None,
    }
  }

  pub fn into_str(self) -> Option<String> {
    match self {
      Self::Str(s) => Some(s),
      _ => None,
    }
  }

  pub fn as_edged(&self) -> Option<&HashMap<String, SerializedNode>> {
    match self {
      Self::Edged(map) => Some(map),
      _ => None,
    }
  }

  pub fn into_edged(self) -> Option<HashMap<String, SerializedNode>> {
    match self {
      Self::Edged(map) => Some(map),
      _ => None,
    }
  }

  pub fn get_path(&self, path: &[&str]) -> Option<&SerializedNode> {
    let mut node = self;
    let mut path = path;
    while !path.is_empty() {
      node = node.as_edged()?.get(path[0])?;
      path = &path[1..];
    }
    Some(node)
  }

  pub fn get_path_str(&self, path: &[&str]) -> Option<&str> {
    self.get_path(path).and_then(Self::as_str)
  }

  /// Create a SerializedNode from a NodeArena and NodeIdx
  pub fn from_node(arena: &NodeArena, idx: NodeIdx) -> Option<Self> {
    let idx = arena.dereference(idx);
    match arena.get(idx) {
      Node::Forwarded(_) => panic!("unexpected forward after dereference"),
      Node::Top => None,
      Node::Str(s) => Some(SerializedNode::Str(s.to_string())),
      Node::Edged(edges) => {
        let mut map: HashMap<String, SerializedNode> = HashMap::new();
        for (k, v) in edges.iter() {
          let value = Self::from_node(arena, *v);
          if let Some(value) = value {
            map.insert(k.to_string(), value);
          }
        }
        if map.is_empty() {
          None
        } else {
          Some(SerializedNode::Edged(map))
        }
      }
    }
  }
}

impl From<&str> for SerializedNode {
  fn from(s: &str) -> Self {
    s.to_string().into()
  }
}

impl From<String> for SerializedNode {
  fn from(s: String) -> Self {
    Self::Str(s)
  }
}

impl From<HashMap<String, SerializedNode>> for SerializedNode {
  fn from(hm: HashMap<String, SerializedNode>) -> Self {
    Self::Edged(hm)
  }
}

impl PartialEq for SerializedNode {
  fn eq(&self, other: &Self) -> bool {
    match (&self, &other) {
      (SerializedNode::Str(s1), SerializedNode::Str(s2)) => s1 == s2,
      (SerializedNode::Str(_), SerializedNode::Edged(_))
      | (SerializedNode::Edged(_), SerializedNode::Str(_)) => false,
      (SerializedNode::Edged(m1), &SerializedNode::Edged(m2)) => {
        if m1.len() != m2.len() {
          return false;
        }

        m1.iter().all(|(k, v)| m2.get(k) == Some(v))
      }
    }
  }
}
