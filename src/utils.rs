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
