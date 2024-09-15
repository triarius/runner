/// A trait for `Result` that allows chaining a function that returns a `Result` to the error type.
///
/// # Examples
/// If the Ok type, `T` has a default value, and the error type, `E` has some ignorable values, then
/// this trait can be used to map the ignorabl error type values to the default value of `T`.
pub(crate) trait FlatMapErr<T, D, E> {
    fn flat_map_err<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(D) -> Result<T, E>;
}

impl<T, D, E> FlatMapErr<T, D, E> for Result<T, D> {
    fn flat_map_err<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(D) -> Result<T, E>,
    {
        match self.map_err(f) {
            Ok(t) | Err(Ok(t)) => Ok(t),
            Err(Err(e)) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn flat_map_err() {
        use super::FlatMapErr;
        use pretty_assertions::assert_eq;

        #[derive(Debug, PartialEq, Eq)]
        enum Error {
            IgnorableError,
            GraveError,
        }

        let ignore = |e| match e {
            Error::IgnorableError => Ok(()),
            Error::GraveError => Err(e),
        };

        [
            (Ok(()), Ok(())),
            (Err(Error::IgnorableError), Ok(())),
            (Err(Error::GraveError), Err(Error::GraveError)),
        ]
        .into_iter()
        .for_each(|(input, expected)| {
            let actual = input.flat_map_err(ignore);
            assert_eq!(actual, expected);
        });
    }
}
