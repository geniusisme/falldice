impl<I> IterExt for I
where
  I: Iterator,
{
  /// as itertools' multiple_cartesian_product, but instead of returning
  /// newly allocated vec each iterations, takes lambda which takes iterator of
  /// items in current element of cartesian product.
  /// It is like doing .map(|vec| vec.iter()) on result of multiple_cartesian_product,
  /// but wihout extra allocation
  fn cartesian_collections<F, R>(self, f: F) -> CartesianProductCollections<I, F>
  where
    Self::Item: Iterator + Clone,
    F: Fn(std::slice::Iter<'_, <<Self as Iterator>::Item as Iterator>::Item>) -> R,
  {
    CartesianProductCollections::<I, F>::new(self, f)
  }
}

pub struct CartesianProductCollections<I, F>
where
  I: Iterator,
  I::Item: Iterator,
{
  items: Items<I>,
  f: F,
}

impl<I, F> CartesianProductCollections<I, F>
where
  I: Iterator,
  I::Item: Iterator,
{
  fn new(iter: I, f: F) -> Self {
    CartesianProductCollections {
      items: Items::InIterator(iter),
      f,
    }
  }
}

impl<I, F, R> Iterator for CartesianProductCollections<I, F>
where
  I: Iterator,
  I::Item: Iterator + Clone,
  F: Fn(std::slice::Iter<'_, <I::Item as Iterator>::Item>) -> R,
{
  type Item = R;

  fn next(&mut self) -> Option<R> {
    let mut old_items = Items::Empty;
    std::mem::swap(&mut old_items, &mut self.items);
    self.items = match old_items {
      Items::InIterator(iter) => {
        let original_iters = iter.collect::<Vec<_>>();
        let mut current_iters = original_iters.clone();
        let current_items = current_iters
          .iter_mut()
          .filter_map(|item_iter| item_iter.next())
          .collect::<Vec<_>>();
        if current_items.len() == current_iters.len() {
          Items::Evaluated(Gathering {
            current_iters,
            original_iters,
            current_items,
          })
        } else {
          Items::Empty
        }
      }
      Items::Evaluated(mut items) => {
        let first_with_item = (0..items.current_iters.len())
          .map(|idx| (idx, items.current_iters[idx].next()))
          .filter_map(|(idx, item)| item.map(|it| (idx, it)))
          .nth(0);

        match first_with_item {
          Some((idx, item)) => {
            items.current_items[idx] = item;
            for i in 0..idx {
              items.current_iters[i] = items.original_iters[i].clone();
              items.current_items[i] = items.current_iters[i].next().unwrap();
            }
            Items::Evaluated(items)
          }
          None => Items::Empty,
        }
      }
      Items::Empty => Items::Empty,
    };
    match self.items {
      Items::Evaluated(ref items) => Some((self.f)(items.current_items.iter())),
      Items::Empty => None,
      _ => unreachable!(),
    }
  }
}

enum Items<I>
where
  I: Iterator,
  I::Item: Iterator,
{
  InIterator(I),
  Evaluated(Gathering<I::Item>),
  Empty,
}

struct Gathering<I: Iterator> {
  current_iters: Vec<I>,
  original_iters: Vec<I>,
  current_items: Vec<I::Item>,
}

pub trait IterExt: Iterator + Sized {
  fn cartesian_collections<F, R>(self, f: F) -> CartesianProductCollections<Self, F>
  where
    Self::Item: Iterator + Clone,
    F: Fn(std::slice::Iter<'_, <<Self as Iterator>::Item as Iterator>::Item>) -> R;
}

#[test]
fn numbers_from_digits() {
  assert_eq!(
    (0..3)
      .map(|i| i..(i + 2))
      .cartesian_collections(|iter| { iter.fold(0, |pr, next| pr * 10 + *next) })
      .collect::<Vec<_>>(),
    vec![12, 112, 22, 122, 13, 113, 23, 123],
  );
}

#[test]
fn if_one_iter_is_empty_result_is_empty() {
  let numbers = vec![vec![1, 2], vec![], vec![1, 2, 3]];
  assert_eq!(
    numbers
      .iter()
      .map(|vec| vec.iter())
      .cartesian_collections(|iter| iter.fold(0, |_, _| 0))
      .next(),
    None,
  )
}
