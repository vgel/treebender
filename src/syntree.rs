use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Constituent<T> {
  pub value: T,
  pub span: (usize, usize),
}

impl<T> fmt::Display for Constituent<T>
where
  T: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}: {}", self.span.0, self.span.1, self.value)
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Word<U> {
  pub value: U,
  pub span: (usize, usize),
}

impl<U> fmt::Display for Word<U>
where
  U: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}: {}", self.span.0, self.span.1, self.value)
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SynTree<T, U> {
  Branch(Constituent<T>, Vec<SynTree<T, U>>),
  Leaf(Word<U>),
}

impl<T, U> SynTree<T, U> {
  pub fn is_leaf(&self) -> bool {
    match self {
      Self::Leaf(_) => true,
      _ => false,
    }
  }

  pub fn is_branch(&self) -> bool {
    match self {
      Self::Branch(_, _) => true,
      _ => false,
    }
  }

  pub fn get_leaf(&self) -> Option<&Word<U>> {
    match self {
      Self::Leaf(w) => Some(w),
      _ => None,
    }
  }

  pub fn get_branch(&self) -> Option<(&Constituent<T>, &Vec<SynTree<T, U>>)> {
    match self {
      Self::Branch(c, cs) => Some((c, cs)),
      _ => None,
    }
  }

  pub fn into_branch(self) -> Option<(Constituent<T>, Vec<SynTree<T, U>>)> {
    match self {
      Self::Branch(c, cs) => Some((c, cs)),
      _ => None,
    }
  }

  pub fn map<V, W>(
    &self,
    map_branch: fn(&Constituent<T>) -> V,
    map_leaf: fn(&Word<U>) -> W,
  ) -> SynTree<V, W> {
    match self {
      Self::Branch(t, children) => {
        let children = children
          .iter()
          .map(|c| c.map(map_branch, map_leaf))
          .collect::<Vec<_>>();
        SynTree::Branch(
          Constituent {
            span: t.span,
            value: map_branch(&t),
          },
          children,
        )
      }
      Self::Leaf(u) => SynTree::Leaf(Word {
        span: u.span,
        value: map_leaf(u),
      }),
    }
  }
}

impl<T, U> fmt::Display for SynTree<T, U>
where
  T: fmt::Display,
  U: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Leaf(t) => write!(f, "{}", t),
      Self::Branch(t, ts) => {
        write!(f, "({}", t)?;
        if ts.len() == 1 {
          write!(f, " ({}))", ts[0])
        } else {
          for t in ts.iter() {
            // TODO: is there a nice way to do this that doesn't allocate a String?
            let fmt = format!("{}", t);
            for line in fmt.lines() {
              write!(f, "\n  {}", line)?;
            }
          }
          write!(f, ")")
        }
      }
    }
  }
}
