mod node;
mod serialized;

pub use node::{Feature, NodeRef};
pub use serialized::SerializedNode;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_construct_fs() {
    let root = NodeRef::new_from_paths(vec![
      Feature {
        path: "a.b".to_string(),
        tag: Some("1".to_string()),
        value: NodeRef::new_top(),
      },
      Feature {
        path: "a.b.c".to_string(),
        tag: None,
        value: NodeRef::new_str("foo".to_string()),
      },
      Feature {
        path: "a.b.d".to_string(),
        tag: None,
        value: NodeRef::new_str("bar".to_string()),
      },
      Feature {
        path: "e".to_string(),
        tag: Some("1".to_string()),
        value: NodeRef::new_top(),
      },
    ])
    .unwrap();

    println!("{}", root);
  }

  #[test]
  fn test_unify_tags() {
    let fs1 = NodeRef::new_from_paths(vec![
      Feature {
        path: "a.b".to_string(),
        tag: Some("1".to_string()),
        value: NodeRef::new_top(),
      },
      Feature {
        path: "c".to_string(),
        tag: Some("1".to_string()),
        value: NodeRef::new_top(),
      },
    ])
    .unwrap();

    let fs2 = NodeRef::new_from_paths(vec![Feature {
      path: "c".to_string(),
      tag: None,
      value: NodeRef::new_str("foo".to_string()),
    }])
    .unwrap();

    // everything is **top** so goes away
    assert!(Option::<SerializedNode>::from(&fs1) == None);

    let gold = SerializedNode::Edged(vec![("c".into(), "foo".into())].into_iter().collect());

    assert!(Option::<SerializedNode>::from(&fs2) == Some(gold));

    NodeRef::unify(fs1.clone(), fs2.clone()).unwrap();

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

    assert!(Option::<SerializedNode>::from(&fs1) == Some(gold.clone()));
    assert!(Option::<SerializedNode>::from(&fs2) == Some(gold));
  }
}
