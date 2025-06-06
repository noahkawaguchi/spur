/// Transforms a `Vec<T>` into a `Vec<U>`.
pub fn vec_into<T, U>(v: Vec<T>) -> Vec<U>
where T: Into<U> {
    v.into_iter().map(Into::into).collect()
}
