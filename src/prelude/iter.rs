use std::{fmt::Debug as DebugT,
          iter::{FromIterator, Iterator}};

pub trait CollectResults<A: DebugT, E: DebugT>
  where Self: Sized
{
  fn collect_results<As: FromIterator<A>, Es: FromIterator<E>>(
    self)
    -> Result<As, Es>;
}

impl<T: Iterator<Item = Result<A, E>>, A: DebugT, E: DebugT>
  CollectResults<A, E> for T
{
  fn collect_results<As: FromIterator<A>, Es: FromIterator<E>>(
    self)
    -> Result<As, Es> {
    let (as_, es_): (Vec<_>, Vec<_>) = self.partition(Result::is_ok);
    if es_.len() > 0 {
      Err(es_.into_iter().map(Result::unwrap_err).collect::<Es>())
    } else {
      Ok(as_.into_iter().map(Result::unwrap).collect::<As>())
    }
  }
}
