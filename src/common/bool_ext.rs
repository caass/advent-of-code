pub(crate) trait BoolExt {
    fn into_num<N: From<u8>>(self) -> N;
}

impl BoolExt for bool {
    /// Convert a `bool` into a numeric type where `false` is 0 and `true` is 1.
    #[inline(always)]
    fn into_num<N: From<u8>>(self) -> N {
        (self as u8).into()
    }
}
