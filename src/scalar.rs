use std::fmt;

use serde_json::Number;

/// A leaf JSON value reached by a dotted path.
///
/// Objects and arrays are not leaves. [`crate::JsonPaths`] walks them and
/// records only the scalars inside. [`Self::Undefined`] appears only in a
/// diff, for a path that exists on one side.
///
/// Integer `1` and decimal `1.0` compare equal. Two integers compare exactly,
/// so a pair of large integers that would collapse to the same float stay
/// different.
#[derive(Debug, Clone)]
pub enum JsonScalar {
    /// JSON `null`.
    Null,
    /// JSON `true` or `false`.
    Bool(bool),
    /// JSON number.
    Number(Number),
    /// JSON string.
    String(String),
    /// The path is missing on this side of a diff.
    Undefined,
}

impl JsonScalar {
    /// A JSON number from an integer.
    #[must_use]
    pub fn from_integer(value: i64) -> Self {
        Self::Number(Number::from(value))
    }

    /// A JSON number from a finite float.
    ///
    /// Returns [`None`] for NaN and infinity, which are not JSON numbers.
    #[must_use]
    pub fn from_float(value: f64) -> Option<Self> {
        Number::from_f64(value).map(Self::Number)
    }
}

impl PartialEq for JsonScalar {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => numbers_equal(left, right),
            _ => false,
        }
    }
}

impl fmt::Display for JsonScalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("null"),
            Self::Bool(value) => write!(formatter, "{value}"),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "{value:?}"),
            Self::Undefined => formatter.write_str("undefined"),
        }
    }
}

impl From<bool> for JsonScalar {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&str> for JsonScalar {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for JsonScalar {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

fn numbers_equal(left: &Number, right: &Number) -> bool {
    if let (Some(left), Some(right)) = (left.as_i64(), right.as_i64()) {
        return left == right;
    }
    if let (Some(left), Some(right)) = (left.as_u64(), right.as_u64()) {
        return left == right;
    }
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}
