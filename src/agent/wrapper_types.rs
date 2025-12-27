use {
    serde::{Deserialize, Serialize},
    std::{borrow::Borrow, hash::Hash, ops::Deref},
};

/// Subject of the tool name change. For tools in mcp servers, you would need to
/// prefix them with their server names
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct OriginalToolName(pub String);

impl Deref for OriginalToolName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<str> for OriginalToolName {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_tool_name_deref() {
        let name = OriginalToolName("test".into());
        assert_eq!(&*name, "test");
    }

    #[test]
    fn original_tool_name_borrow() {
        let name = OriginalToolName("test".into());
        let borrowed: &str = name.borrow();
        assert_eq!(borrowed, "test");
    }

    #[test]
    fn original_tool_name_serde() {
        let name = OriginalToolName("test".into());
        let json = serde_json::to_string(&name).unwrap();
        let deserialized: OriginalToolName = serde_json::from_str(&json).unwrap();
        assert_eq!(name, deserialized);
    }
}
