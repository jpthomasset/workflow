// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8e12aab3d9eb9097b718a8bc6453c5f9
pub trait AdaptErr<T, E0, E1>
where
    E1: From<E0>,
{
    /// Convert the error in a result to the infered type.
    ///
    /// Allowing to go from `Result<T, E0>` to `Result<T, E1>` given there is a conversion available between `E0` and `E1`
    ///
    fn adapt(self) -> Result<T, E1>
    where
        Self: Into<Result<T, E0>>, // Ensure we are given something that looks like a Result by ensuring we have a conversion to it
                                   // Most of the time, the conversion will be `From<T> for T` because we will get a Result directly
    {
        Into::<Result<T, E0>>::into(self).map_err(|e| e.into())
    }
}

impl<T, E0, E1> AdaptErr<T, E0, E1> for Result<T, E0> where E1: From<E0> {}
