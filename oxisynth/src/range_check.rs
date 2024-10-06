pub(crate) fn range_check<E, T: PartialOrd, C: std::ops::RangeBounds<T>>(
    range: C,
    value: &T,
    error: E,
) -> Result<(), E> {
    if range.contains(value) {
        Ok(())
    } else {
        Err(error)
    }
}
