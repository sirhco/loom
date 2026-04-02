use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;

/// Controls how the permission system handles tool execution requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PermissionMode {
    /// Ask for dangerous operations, auto-approve safe reads.
    #[default]
    Default,
    /// Auto-approve safe operations (reads and non-destructive writes).
    Auto,
    /// Approve everything without prompting.
    Bypass,
}

impl fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionMode::Default => write!(f, "default"),
            PermissionMode::Auto => write!(f, "auto"),
            PermissionMode::Bypass => write!(f, "bypass"),
        }
    }
}

impl FromStr for PermissionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(PermissionMode::Default),
            "auto" => Ok(PermissionMode::Auto),
            "bypass" => Ok(PermissionMode::Bypass),
            other => Err(format!(
                "invalid permission mode '{}': expected one of 'default', 'auto', 'bypass'",
                other
            )),
        }
    }
}

impl ValueEnum for PermissionMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            PermissionMode::Default,
            PermissionMode::Auto,
            PermissionMode::Bypass,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            PermissionMode::Default => {
                Some(clap::builder::PossibleValue::new("default").help("Ask for dangerous ops, auto-approve safe reads"))
            }
            PermissionMode::Auto => {
                Some(clap::builder::PossibleValue::new("auto").help("Auto-approve safe operations"))
            }
            PermissionMode::Bypass => {
                Some(clap::builder::PossibleValue::new("bypass").help("Approve everything without prompting"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(PermissionMode::Default.to_string(), "default");
        assert_eq!(PermissionMode::Auto.to_string(), "auto");
        assert_eq!(PermissionMode::Bypass.to_string(), "bypass");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("default".parse::<PermissionMode>().unwrap(), PermissionMode::Default);
        assert_eq!("AUTO".parse::<PermissionMode>().unwrap(), PermissionMode::Auto);
        assert_eq!("Bypass".parse::<PermissionMode>().unwrap(), PermissionMode::Bypass);
        assert!("invalid".parse::<PermissionMode>().is_err());
    }

    #[test]
    fn test_default() {
        assert_eq!(PermissionMode::default(), PermissionMode::Default);
    }
}
