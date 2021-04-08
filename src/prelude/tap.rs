pub trait Tap<T> {
  fn tap(self, action: impl FnOnce(&T) -> ()) -> Self;
}

pub trait TapMut<T> {
  fn tap_mut(self, action: impl FnMut(&mut T) -> ()) -> Self;
}

pub trait TapErr<E> {
  fn tap_err(self, action: impl FnOnce(&E) -> ()) -> Self;
}

pub trait TapErrMut<E> {
  fn tap_err_mut(self, action: impl FnOnce(&mut E) -> ()) -> Self;
}

// Tap impls
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

// TapErr impls
impl<T, E> TapErr<E> for Result<T, E> {
  fn tap_err(self, action: impl FnOnce(&E) -> ()) -> Self {
    self.map_err(|v| {
          action(&v);
          v
        })
  }
}

// TapErrMut impls
impl<T, E> TapErrMut<E> for Result<T, E> {
  fn tap_err_mut(self, action: impl FnOnce(&mut E) -> ()) -> Self {
    self.map_err(|mut v| {
          action(&mut v);
          v
        })
  }
}

// TapMut impls
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
