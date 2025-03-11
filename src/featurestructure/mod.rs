mod node;
mod serialized;

pub use node::{Feature, NodeArena, NodeIdx};
pub use serialized::SerializedNode;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_construct_fs() {
    let mut arena = NodeArena::new();

    let features = vec![
      Feature {
        path: "a.b".to_string(),
        tag: Some("1".to_string()),
        value: arena.alloc_top(),
      },
      Feature {
        path: "a.b.c".to_string(),
        tag: None,
        value: arena.alloc_str("foo".to_string()),
      },
      Feature {
        path: "a.b.d".to_string(),
        tag: None,
        value: arena.alloc_str("bar".to_string()),
      },
      Feature {
        path: "e".to_string(),
        tag: Some("1".to_string()),
        value: arena.alloc_top(),
      },
    ];

    let root = arena.alloc_from_features(features).unwrap();

    println!("{}", arena.display(root));
  }

  #[test]
  fn test_unify_tags() {
    let mut arena = NodeArena::new();

    let features1 = vec![
      Feature {
        path: "a.b".to_string(),
        tag: Some("1".to_string()),
        value: arena.alloc_top(),
      },
      Feature {
        path: "c".to_string(),
        tag: Some("1".to_string()),
        value: arena.alloc_top(),
      },
    ];

    let fs1 = arena.alloc_from_features(features1).unwrap();

    let features2 = vec![Feature {
      path: "c".to_string(),
      tag: None,
      value: arena.alloc_str("foo".to_string()),
    }];

    let fs2 = arena.alloc_from_features(features2).unwrap();

    // everything is **top** so goes away
    assert!(SerializedNode::from_node(&arena, fs1).is_none());

    let gold = SerializedNode::Edged(vec![("c".into(), "foo".into())].into_iter().collect());

    assert!(SerializedNode::from_node(&arena, fs2) == Some(gold));

    arena.unify(fs1, fs2).unwrap();

    let gold = SerializedNode::Edged(
      vec![
        (
          "a".into(),
          SerializedNode::Edged(vec![("b".into(), "foo".into())].into_iter().collect()),
        ),
        ("c".into(), "foo".into()),
      ]
      .into_iter()
      .collect(),
    );

    assert!(SerializedNode::from_node(&arena, fs1) == Some(gold.clone()));
    assert!(SerializedNode::from_node(&arena, fs2) == Some(gold));
  }
}
