/// A trait for concisely converting a `Vec<T>` into a `Vec<U>` so that `.map_into()` can be called
/// in the same manner as `.into()`.
pub trait MapInto<T, U>
where T: Into<U>
{
    /// Transforms a `Vec<T>` into a `Vec<U>`.
    fn map_into(self) -> Vec<U>;
}

impl<T, U> MapInto<T, U> for Vec<T>
where T: Into<U>
{
    fn map_into(self) -> Vec<U> { self.into_iter().map(Into::into).collect() }
}
