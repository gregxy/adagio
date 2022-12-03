#[macro_export]
macro_rules! create_msg_err {
    ($err_name:ident) => {
        pub struct $err_name {
            message: String,
        }

        impl std::fmt::Display for $err_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($err_name), self.message)
            }
        }

        impl std::fmt::Debug for $err_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($err_name), self.message)
            }
        }

        impl std::error::Error for $err_name {}

        impl $err_name {
            pub fn new<S: Into<String>>(msg: S) -> Self {
                Self {
                    message: msg.into(),
                }
            }
        }
    };
}

create_msg_err!(AssertionError);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertion_error_to_string() {
        let err = AssertionError::new("foo and bar");

        assert_eq!(err.to_string(), "AssertionError: foo and bar".to_string());
    }
}
