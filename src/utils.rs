trait Tap<T> {
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

trait TapMut<T> {
  fn tap_mut(&mut self, action: impl FnOnce(&mut T) -> ());
}

impl<T, E> TapMut<T> for Result<T, E> {
  fn tap_mut(self, action: impl FnOnce(&mut T) -> ()) -> Self {
    self.map(|v| {
      action(&mut v);
      v
    })
  }
}

impl<T> TapMut<T> for Option<T> {
  fn tap_mut(self, action: impl FnOnce(&mut T) -> ()) -> Self {
    self.map(|v| {
      action(&mut v);
      v
    })
  }
}

