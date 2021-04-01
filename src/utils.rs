pub trait Monad<A, B>
where
  Self: Sized,
{
  type Out;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out;
}

pub trait Functor<A, B> {
  type Out;
  fn fmap(self, f: impl FnOnce(A) -> B) -> Self::Out;
}

pub trait BiFunctor<A1, A2, B1, B2> {
  type Out;
  fn bi_map(
    self,
    f1: impl FnOnce(A1) -> B1,
    f2: impl FnOnce(A2) -> B2,
  ) -> Self::Out;
}

impl<A, B, E> Monad<A, B> for Result<A, E> {
  type Out = Result<B, E>;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out {
    self.and_then(f)
  }
}

impl<A, B> Monad<A, B> for Option<A> {
  type Out = Option<B>;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out {
    self.and_then(f)
  }
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

pub trait NormalizeResult<T> {
  fn norm(self) -> Result<T, crate::AnyError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> NormalizeResult<T>
  for Result<T, E>
{
  fn norm(self) -> Result<T, crate::AnyError> {
    self.map_err(|e| Box::new(e) as crate::AnyError)
  }
}

pub trait Tap<T> {
  fn tap(self, action: impl FnOnce(&T) -> ()) -> Self;
}

impl<T, E> Tap<T> for Result<T, E> {
  fn tap(self, action: impl FnOnce(&T) -> ()) -> Self {
    self.map(|v| {
      action(&v);
      v
    })
  }
}

impl<T> Tap<T> for Option<T> {
  fn tap(self, action: impl FnOnce(&T) -> ()) -> Self {
    self.map(|v| {
      action(&v);
      v
    })
  }
}

pub trait TapMut<T> {
  fn tap_mut(self, action: impl FnMut(&mut T) -> ()) -> Self;
}

impl<T, E> TapMut<T> for Result<T, E> {
  fn tap_mut(self, mut action: impl FnMut(&mut T) -> ()) -> Self {
    self.map(|mut v| {
      action(&mut v);
      v
    })
  }
}

impl<T> TapMut<T> for Option<T> {
  fn tap_mut(self, action: impl FnOnce(&mut T) -> ()) -> Self {
    self.map(|mut v| {
      action(&mut v);
      v
    })
  }
}
