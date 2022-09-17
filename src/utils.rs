use std::error::Error;

/// Boxed static error type
pub type Err = Box<dyn Error + 'static>;

/// Takes a list where each element is a set of choices, and returns all the possible sets
/// generated. Will clone the elements.
///
/// ```
/// let v = vec![
///   vec![1],
///   vec![2, 3],
///   vec![4],
///   vec![5, 6, 7],
/// ];
///
/// assert_eq!(treebender::utils::combinations(&v), vec![
///   vec![1, 2, 4, 5],
///   vec![1, 3, 4, 5],
///   vec![1, 2, 4, 6],
///   vec![1, 3, 4, 6],
///   vec![1, 2, 4, 7],
///   vec![1, 3, 4, 7],
/// ]);
/// ```
pub fn combinations<T>(list: &[Vec<T>]) -> Vec<Vec<T>>
where
  T: Clone,
{
  if list.is_empty() {
    Vec::new()
  } else if list.len() == 1 {
    list[0].iter().map(|e| vec![e.clone()]).collect()
  } else {
    let (head, tail) = list.split_at(1);
    let head = &head[0];

    combinations(tail)
      .into_iter()
      .flat_map(|subseq| {
        // prepend every element of the head to every possible subseq
        head.iter().map(move |v| {
          let mut newseq = subseq.clone();
          newseq.insert(0, v.clone());
          newseq
        })
      })
      .collect()
  }
}
