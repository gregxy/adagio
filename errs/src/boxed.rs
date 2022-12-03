use std::any::TypeId;
use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub struct BoxedError {
    inner: Box<dyn std::error::Error + 'static>,
    type_id: TypeId,
    detail_message: Option<String>,
}

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail_message {
            Some(message) => write!(f, "Error: {}\nCaused by: {}", message, self.inner),
            None => write!(f, "{}", self.inner),
        }
    }
}

impl<E> From<E> for BoxedError
where
    E: std::error::Error + 'static,
{
    fn from(e: E) -> Self {
        Self {
            inner: Box::new(e),
            type_id: TypeId::of::<E>(),
            detail_message: None,
        }
    }
}

impl From<BoxedError> for Box<dyn std::error::Error> {
    fn from(e: BoxedError) -> Self {
        e.inner
    }
}

impl BoxedError {
    pub fn new<E>(e: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            inner: Box::from(e),
            type_id: TypeId::of::<E>(),
            detail_message: None,
        }
    }

    pub fn with_detail_message<E, S>(e: E, msg: S) -> Self
    where
        S: Into<String>,
        E: std::error::Error + 'static,
    {
        Self {
            inner: Box::from(e),
            type_id: TypeId::of::<E>(),
            detail_message: Some(msg.into()),
        }
    }

    pub fn is<T: std::error::Error + 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    pub fn detail_message(&self) -> Option<&str> {
        match &self.detail_message {
            Some(message) => Some(message.as_str()),
            None => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, BoxedError>;
