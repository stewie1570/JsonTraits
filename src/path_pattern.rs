/// Matches dotted paths against patterns.
///
/// A pattern segment of `*` matches exactly one path segment.
/// `contacts.*.info.name` matches `contacts.0.info.name` and does not match
/// `contacts.0.info.name.last`.
///
/// These methods correspond to `IsSupportedBy` and `IsAPathMatchWith`.
/// Call [`PathPattern::is_supported_by`] on a path, and
/// [`PathPattern::is_a_path_match_with`] on a pattern.
pub trait PathPattern {
    /// `true` when this path matches any of `patterns`.
    ///
    /// # Examples
    ///
    /// ```
    /// use json_traits::PathPattern;
    ///
    /// let patterns = ["prop1.prop2", "contacts.*.info.name", "contacts.*.info.number"];
    /// assert!("contacts.0.info.name".is_supported_by(patterns));
    /// assert!(!"contacts.0.info.name.last".is_supported_by(patterns));
    /// ```
    #[doc(alias = "IsSupportedBy")]
    #[must_use]
    fn is_supported_by<I, S>(&self, patterns: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>;

    /// `true` when this pattern matches `path`.
    ///
    /// The receiver is the pattern: `"contacts.*.name".is_a_path_match_with("contacts.0.name")`.
    #[doc(alias = "IsAPathMatchWith")]
    #[must_use]
    fn is_a_path_match_with(&self, path: &str) -> bool;
}

impl PathPattern for str {
    fn is_supported_by<I, S>(&self, patterns: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        patterns
            .into_iter()
            .any(|pattern| pattern.as_ref().is_a_path_match_with(self))
    }

    fn is_a_path_match_with(&self, path: &str) -> bool {
        let mut pattern_parts = self.split('.');
        let mut path_parts = path.split('.');
        loop {
            match (pattern_parts.next(), path_parts.next()) {
                (None, None) => return true,
                (Some("*"), Some(_)) => {}
                (Some(pattern_part), Some(path_part)) if pattern_part == path_part => {}
                _ => return false,
            }
        }
    }
}
