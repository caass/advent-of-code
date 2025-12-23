use eyre::Report;

pub trait Solution: Sync {
    fn solve(&self, input: &str) -> Result<String, Report>;
}

impl<F, R> Solution for F
where
    F: Fn(&str) -> R + Sync,
    R: ReturnValue,
{
    fn solve(&self, input: &str) -> Result<String, Report> {
        (self)(input.trim_end()).into_result()
    }
}

pub trait ReturnValue: Sized {
    fn into_result(self) -> Result<String, Report>;
}

impl<T: ToString, E: Into<Report>> ReturnValue for Result<T, E> {
    fn into_result(self) -> Result<String, Report> {
        match self {
            Ok(t) => Ok(t.to_string()),
            Err(e) => Err(e.into()),
        }
    }
}

macro_rules! impl_return_value_for {
    ($($ty:ty),+) => {
        $(
            impl ReturnValue for $ty {
                fn into_result(self) -> Result<String, Report> {
                    Ok(self.to_string())
                }
            }
        )+
    };
}

impl_return_value_for!(usize, isize);

impl ReturnValue for () {
    fn into_result(self) -> Result<String, Report> {
        unimplemented!()
    }
}
