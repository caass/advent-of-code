use std::str::FromStr;

pub(crate) trait TryFromStr<'s>: Sized {
    type Err;

    fn try_from_str(s: &'s str) -> Result<Self, Self::Err>;
}

impl<T: FromStr> TryFromStr<'_> for T {
    type Err = <Self as FromStr>::Err;

    fn try_from_str(s: &'_ str) -> Result<Self, Self::Err> {
        T::from_str(s)
    }
}

pub(crate) trait TryParse {
    fn try_parse<'a, T: TryFromStr<'a>>(&'a self) -> Result<T, T::Err>;
}

impl TryParse for str {
    fn try_parse<'a, T: TryFromStr<'a>>(&'a self) -> Result<T, T::Err> {
        T::try_from_str(self)
    }
}
