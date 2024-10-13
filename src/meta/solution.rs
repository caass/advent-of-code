use eyre::Report;

pub trait Solution {
    fn solve(&self, input: &str) -> Result<String, Report>;
}

impl<F, T, E> Solution for F
where
    F: Fn(&str) -> Result<T, E>,
    T: ToString,
    E: Into<Report>,
{
    fn solve(&self, input: &str) -> Result<String, Report> {
        (self)(input)
            .map(|output| output.to_string())
            .map_err(E::into)
    }
}
