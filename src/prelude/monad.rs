use std::future::Future;

use async_trait::async_trait;

pub trait Monad<A, B>
  where Self: Sized
{
  type Out;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out;
}

impl<A, B> Monad<A, B> for Option<A> {
  type Out = Option<B>;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out {
    self.and_then(f)
  }
}

impl<A, B, E> Monad<A, B> for Result<A, E> {
  type Out = Result<B, E>;
  fn bind(self, f: impl FnOnce(A) -> Self::Out) -> Self::Out {
    self.and_then(f)
  }
}

pub trait MonadErr<E, B>
  where Self: Sized
{
  type Out;
  fn bind_err(self, f: impl FnOnce(E) -> Self::Out) -> Self::Out;
}

impl<A, E, B> MonadErr<E, B> for Result<A, E> {
  type Out = Result<A, B>;
  fn bind_err(self, f: impl FnOnce(E) -> Self::Out) -> Self::Out {
    match self {
      | Ok(ok) => Ok(ok),
      | Err(err) => f(err),
    }
  }
}

#[async_trait]
pub trait MonadAsync<A, B>
  where Self: Sized + Send
{
  type Out;
  async fn bind_async<F: Send + FnOnce(A) -> O,
                        O: Send + Future<Output = Self::Out>>(
    self,
    f: F)
    -> Self::Out;
}

#[async_trait]
impl<A: Send, B: Send, E: Send> MonadAsync<A, B> for Result<A, E> {
  type Out = Result<B, E>;
  async fn bind_async<F: Send + FnOnce(A) -> O,
                        O: Send + Future<Output = Self::Out>>(
    self,
    f: F)
    -> Self::Out {
    match self {
      | Ok(v) => f(v).await,
      | Err(e) => Err(e),
    }
  }
}
