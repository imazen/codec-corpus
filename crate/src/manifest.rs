use serde::Deserialize;

/// Expected behavior when a codec processes this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpectedBehavior {
    /// Codec must decode without crashing (may produce errors).
    MustNotCrash,
    /// Codec should decode successfully.
    ShouldDecode,
    /// Behavior is unknown / not yet classified.
    #[default]
    Unknown,
}

/// Information about the GitHub issue this blob originated from.
#[derive(Debug, Clone, Deserialize)]
pub struct IssueInfo {
    pub repo: String,
    pub category: String,
    pub number: u64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// A single entry in the blob manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestEntry {
    /// SHA-256 hex digest (content address).
    pub sha256: String,
    /// Detected or declared image format (e.g. "png", "webp"), if known.
    pub format: Option<String>,
    /// File size in bytes.
    pub file_size: u64,
    /// How this blob was collected (e.g. "github-issues", "built", "internet").
    pub source: String,
    /// Finer-grained source label (e.g. "corpus/png-8", "scraping/avif").
    pub source_label: String,
    /// What we expect a codec to do with this file.
    #[serde(default)]
    pub expected_behavior: ExpectedBehavior,
    /// Bug classification (e.g. "crash", "oom", "other").
    #[serde(default)]
    pub bug_type: Option<String>,
    /// Confidence score for the classification (0.0–1.0).
    #[serde(default)]
    pub confidence: f64,
    /// GitHub issue metadata, present only for `source = "github-issues"`.
    #[serde(default)]
    pub issue: Option<IssueInfo>,
}

/// Parse a JSONL manifest into a vector of entries.
///
/// Lines that fail to parse are silently skipped (the manifest may evolve
/// to include new fields; we don't want old crate versions to hard-fail).
pub fn parse_manifest(text: &str) -> Vec<ManifestEntry> {
    text.lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"sha256":"000017835ad044b8c06908dfb85cb0df3ba546c67defc877de65f6ae4c9fb33f","format":null,"file_size":807,"source":"github-issues","source_label":"github-issues","expected_behavior":"unknown","bug_type":"other","confidence":0.0,"issue":{"repo":"ImageMagick/ImageMagick","category":"imagemagick","number":4504,"title":"SVG to PNG conversion","state":"CLOSED","labels":[]}}
{"sha256":"0000e95f95de5cb1d4cb1a7fb327b7fbdf14094b9470e27c8593062f495d7374","format":"png","file_size":82105,"source":"built","source_label":"corpus/png-8"}
{"sha256":"00026ff25efe23b8e7a5aec221f91f4269ef4b3a4df694d2c6661cb6aac777e2","format":"avif","file_size":9114,"source":"internet","source_label":"scraping/avif"}"#;

    #[test]
    fn parse_sample_lines() {
        let entries = parse_manifest(SAMPLE);
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].source, "github-issues");
        assert!(entries[0].issue.is_some());
        assert_eq!(entries[0].expected_behavior, ExpectedBehavior::Unknown);
        assert_eq!(entries[0].file_size, 807);

        assert_eq!(entries[1].format.as_deref(), Some("png"));
        assert_eq!(entries[1].source_label, "corpus/png-8");
        assert!(entries[1].issue.is_none());

        assert_eq!(entries[2].format.as_deref(), Some("avif"));
        assert_eq!(entries[2].file_size, 9114);
    }

    #[test]
    fn parse_empty() {
        assert!(parse_manifest("").is_empty());
        assert!(parse_manifest("\n\n").is_empty());
    }

    #[test]
    fn parse_skips_bad_lines() {
        let input = format!("{}\nnot-json\n{}", SAMPLE.lines().next().unwrap(), "{}");
        let entries = parse_manifest(&input);
        // First line parses, "not-json" skipped, "{}" skipped (missing required fields)
        assert_eq!(entries.len(), 1);
    }
}
