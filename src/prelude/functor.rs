pub trait Functor<A, B> {
  type Out;
  fn fmap(self, f: impl FnOnce(A) -> B) -> Self::Out;
}

pub trait BiFunctor<A1, A2, B1, B2> {
  type Out;
  fn bi_map(self,
            f1: impl FnOnce(A1) -> B1,
            f2: impl FnOnce(A2) -> B2)
            -> Self::Out;
}

impl<A, B, E> Functor<A, B> for Result<A, E> {
  type Out = Result<B, E>;
  fn fmap(self, f: impl FnOnce(A) -> B) -> Self::Out {
    self.map(f)
  }
}

impl<A, B> Functor<A, B> for Option<A> {
  type Out = Option<B>;
  fn fmap(self, f: impl FnOnce(A) -> B) -> Self::Out {
    self.map(f)
  }
}
