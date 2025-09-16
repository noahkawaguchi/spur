/// A trait for concisely converting an iterable collection into a collection of `T` so that
/// `.map_into()` can be called in the same manner as `.into()`.
pub trait MapInto<T>: IntoIterator
where Self::Item: Into<T>
{
    /// Transforms `self` into `C`, a collection of `T`.
    fn map_into<C: FromIterator<T>>(self) -> C;
}

impl<T, I> MapInto<T> for I
where
    I: IntoIterator,
    Self::Item: Into<T>,
{
    fn map_into<C: FromIterator<T>>(self) -> C { self.into_iter().map(Into::into).collect() }
}
