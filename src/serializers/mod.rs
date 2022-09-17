pub mod return_operation;
pub mod set_operation;

#[derive(Debug)]
pub struct SerializationError {
    pub message: String,
}

impl std::error::Error for SerializationError {}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)?;
        Ok(())
    }
}

impl serde::ser::Error for SerializationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            message: msg.to_string(),
        }
    }
}
