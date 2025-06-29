use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownLink {
    pub text: String,
    pub target: String,
    pub line_number: usize,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub struct LinkAnalyzer {
    documents: HashMap<PathBuf, Vec<MarkdownLink>>,
}

impl LinkAnalyzer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn extract_links(content: &str) -> Vec<(String, String, usize)> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inline_links() {
        let content = "Check out [Rust](https://www.rust-lang.org) for more info.";
        let links = LinkAnalyzer::extract_links(content);
        
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "Rust");
        assert_eq!(links[0].1, "https://www.rust-lang.org");
        assert_eq!(links[0].2, 1);
    }

    #[test]
    fn test_extract_reference_links() {
        let content = "Check out [Rust][rust-lang] for more info.\n\n[rust-lang]: https://www.rust-lang.org";
        let links = LinkAnalyzer::extract_links(content);
        
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "Rust");
        assert_eq!(links[0].1, "https://www.rust-lang.org");
        assert_eq!(links[0].2, 1);
    }

    #[test]
    fn test_extract_relative_links() {
        let content = "See [documentation](./docs/README.md) for details.";
        let links = LinkAnalyzer::extract_links(content);
        
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "documentation");
        assert_eq!(links[0].1, "./docs/README.md");
        assert_eq!(links[0].2, 1);
    }

    #[test]
    fn test_extract_multiple_links_with_line_numbers() {
        let content = "First [link1](url1)\n\nSecond [link2](url2)\nThird [link3](url3)";
        let links = LinkAnalyzer::extract_links(content);
        
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].2, 1);
        assert_eq!(links[1].2, 3);
        assert_eq!(links[2].2, 4);
    }
}