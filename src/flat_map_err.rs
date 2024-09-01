/// A trait for `Result` that allows chaining a function that returns a `Result` to the error type.
///
/// # Examples
/// One use case is if the ok type, `T` has a default value, and the error type, `E` may take
/// multiple values that may be ignored, then this trait can be used to map the error type to the
/// default value of `T` for the errors that can be ignored.
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

        #[derive(Debug, PartialEq, Eq)]
        enum Error {
            IgnorableError,
            GraveError,
        }

        let ignore = |e| match e {
            Error::IgnorableError => Ok(()),
            Error::GraveError => Err(e),
        };

        let success = Ok(());
        let ignorable_error = Err(Error::IgnorableError);
        let grave_error = Err(Error::GraveError);

        assert_eq!(success.flat_map_err(ignore), Ok(()));
        assert_eq!(ignorable_error.flat_map_err(ignore), Ok(()));
        assert_eq!(grave_error.flat_map_err(ignore), Err(Error::GraveError));
    }
}