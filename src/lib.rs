pub mod adapt_err;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod git;
pub mod init;
pub mod jira;

pub fn repeat_until_ok<F, T, E>(mut f: F) -> T
where
    F: FnMut() -> Result<T, E>,
{
    loop {
        if let Ok(t) = f() {
            return t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_return_result() {
        let res = repeat_until_ok(|| Ok::<i32, String>(1));
        assert_eq!(res, 1);
    }

    #[test]
    fn test_repeat_return_result_eventually() {
        let mut x = 0;
        let res = repeat_until_ok(move || -> Result<i32, String> {
            if x > 5 {
                Ok(x)
            } else {
                x += 1;
                Err("Not ok".to_string())
            }
        });
        assert_eq!(res, 6);
    }
}
