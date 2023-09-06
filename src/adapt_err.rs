pub trait Adapt<T0, T1, E0, E1>
where
    T1: From<T0>,
    E1: From<E0>,
{
    /// Convert the Ok and Err result to the infered type.
    ///
    /// Allowing to go from `Result<T0, E0>` to `Result<T1, E1>` given there is a conversion available between `T0` and `T1` and/or E0 and E1.
    ///
    /// It works also for `Result<T0, E>` to `Result<T1, E>` or `Result<T, E0>` to `Result<T, E1>` because there's always a conversion from
    /// E to E or T to T (`From<T> for T`).
    ///
    fn adapt(self) -> Result<T1, E1>
    where
        Self: Into<Result<T0, E0>>, // Ensure we are given something that looks like a Result by ensuring we have a conversion to it
                                    // Most of the time, the conversion will be `From<T> for T` because we will get a Result directly
    {
        Into::<Result<T0, E0>>::into(self)
            .map(|t| t.into())
            .map_err(|e| e.into())
    }
}

impl<T0, T1, E0, E1> Adapt<T0, T1, E0, E1> for Result<T0, E0>
where
    T1: From<T0>,
    E1: From<E0>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    enum SomeError {
        Yay,
    }

    #[derive(PartialEq, Eq, Debug)]
    enum OtherError {
        Nay,
    }

    impl From<SomeError> for OtherError {
        fn from(_value: SomeError) -> Self {
            OtherError::Nay
        }
    }

    #[test]
    fn test_adapt_can_adapt_err() {
        let a: Result<usize, SomeError> = Err(SomeError::Yay);
        let b: Result<usize, OtherError> = a.adapt();
        assert_eq!(b, Err(OtherError::Nay));
    }

    #[test]
    fn test_adapt_do_not_affect_ok_res() {
        let a: Result<usize, SomeError> = Ok(1);
        let b: Result<usize, OtherError> = a.adapt();
        assert_eq!(b, Ok(1));
    }

    #[test]
    fn test_adapt_can_adapt_ok() {
        let a: Result<i8, SomeError> = Ok(1);
        let b: Result<i16, SomeError> = a.adapt();
        assert_eq!(b, Ok(1));
    }

    #[test]
    fn test_adapt_can_adapt_both() {
        let a: Result<i8, SomeError> = Ok(1);
        let b: Result<i16, OtherError> = a.adapt();
        assert_eq!(b, Ok(1i16));
    }
}
