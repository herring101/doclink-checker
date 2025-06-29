use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

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
    base_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrokenLink {
    pub link: MarkdownLink,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    pub total_links: usize,
    pub internal_links: usize,
    pub external_links: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LinkStatistics {
    pub total_documents: usize,
    pub total_links: usize,
    pub internal_links: usize,
    pub external_links: usize,
    pub broken_links: usize,
    pub orphaned_documents: usize,
    pub document_stats: HashMap<PathBuf, DocumentStats>,
}

impl LinkAnalyzer {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            documents: HashMap::new(),
            base_path,
        }
    }

    pub fn analyze_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for entry in WalkDir::new(&self.base_path) {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(path)?;
                let links = Self::extract_links(&content);

                let markdown_links: Vec<MarkdownLink> = links
                    .into_iter()
                    .map(|(text, target, line_number)| MarkdownLink {
                        text,
                        target,
                        line_number,
                        file_path: path.to_path_buf(),
                    })
                    .collect();

                self.documents.insert(path.to_path_buf(), markdown_links);
            }
        }
        Ok(())
    }

    pub fn find_broken_links(&self) -> Vec<BrokenLink> {
        let mut broken_links = Vec::new();

        for (file_path, links) in &self.documents {
            for link in links {
                if link.target.starts_with("http://") || link.target.starts_with("https://") {
                    continue;
                }

                let resolved_path = if link.target.starts_with('/') {
                    self.base_path.join(&link.target[1..])
                } else {
                    file_path
                        .parent()
                        .unwrap_or(&self.base_path)
                        .join(&link.target)
                };

                let resolved_path = resolved_path.canonicalize().unwrap_or(resolved_path);

                if !resolved_path.exists() {
                    broken_links.push(BrokenLink {
                        link: link.clone(),
                        reason: format!("File not found: {}", resolved_path.display()),
                    });
                }
            }
        }

        broken_links
    }

    pub fn find_orphaned_documents(&self) -> Vec<PathBuf> {
        let mut referenced_docs = HashSet::new();
        referenced_docs.insert(self.base_path.join("README.md"));
        referenced_docs.insert(self.base_path.join("readme.md"));

        for (file_path, links) in &self.documents {
            for link in links {
                if link.target.starts_with("http://") || link.target.starts_with("https://") {
                    continue;
                }

                let resolved_path = if link.target.starts_with('/') {
                    self.base_path.join(&link.target[1..])
                } else {
                    file_path
                        .parent()
                        .unwrap_or(&self.base_path)
                        .join(&link.target)
                };

                if let Ok(canonical_path) = resolved_path.canonicalize() {
                    referenced_docs.insert(canonical_path);
                }
            }
        }

        let mut orphaned = Vec::new();
        for doc_path in self.documents.keys() {
            if let Ok(canonical_path) = doc_path.canonicalize() {
                if !referenced_docs.contains(&canonical_path) {
                    orphaned.push(doc_path.clone());
                }
            }
        }

        orphaned
    }

    pub fn get_statistics(&self) -> LinkStatistics {
        let mut stats = LinkStatistics::default();
        stats.total_documents = self.documents.len();

        let mut all_links = Vec::new();
        for (doc_path, links) in &self.documents {
            stats.total_links += links.len();

            let mut internal_count = 0;
            let mut external_count = 0;

            for link in links {
                if link.target.starts_with("http://") || link.target.starts_with("https://") {
                    external_count += 1;
                } else {
                    internal_count += 1;
                }
                all_links.push(link);
            }

            stats.document_stats.insert(
                doc_path.clone(),
                DocumentStats {
                    total_links: links.len(),
                    internal_links: internal_count,
                    external_links: external_count,
                },
            );
        }

        for link in &all_links {
            if link.target.starts_with("http://") || link.target.starts_with("https://") {
                stats.external_links += 1;
            } else {
                stats.internal_links += 1;
            }
        }

        stats.broken_links = self.find_broken_links().len();
        stats.orphaned_documents = self.find_orphaned_documents().len();

        stats
    }

    pub fn extract_links(content: &str) -> Vec<(String, String, usize)> {
        let mut links = Vec::new();
        let mut reference_definitions = HashMap::new();

        let reference_def_regex = Regex::new(r"^\[([^\]]+)\]:\s*(.+)$").unwrap();
        for (_line_num, line) in content.lines().enumerate() {
            if let Some(caps) = reference_def_regex.captures(line) {
                let label = caps.get(1).unwrap().as_str().to_lowercase();
                let url = caps.get(2).unwrap().as_str().trim();
                reference_definitions.insert(label, url.to_string());
            }
        }

        let inline_link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        let reference_link_regex = Regex::new(r"\[([^\]]+)\]\[([^\]]*)\]").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            for caps in inline_link_regex.captures_iter(line) {
                let text = caps.get(1).unwrap().as_str().to_string();
                let target = caps.get(2).unwrap().as_str().to_string();
                links.push((text, target, line_num + 1));
            }

            for caps in reference_link_regex.captures_iter(line) {
                let text = caps.get(1).unwrap().as_str().to_string();
                let label = caps.get(2).unwrap().as_str();
                let label_key = if label.is_empty() {
                    text.to_lowercase()
                } else {
                    label.to_lowercase()
                };

                if let Some(url) = reference_definitions.get(&label_key) {
                    links.push((text, url.clone(), line_num + 1));
                }
            }
        }

        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

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
        let content =
            "Check out [Rust][rust-lang] for more info.\n\n[rust-lang]: https://www.rust-lang.org";
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

    #[test]
    fn test_find_broken_links() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let doc1_path = base_path.join("doc1.md");
        let mut doc1 = fs::File::create(&doc1_path).unwrap();
        writeln!(doc1, "# Document 1").unwrap();
        writeln!(doc1, "[Valid link](./doc2.md)").unwrap();
        writeln!(doc1, "[Broken link](./nonexistent.md)").unwrap();

        let doc2_path = base_path.join("doc2.md");
        let mut doc2 = fs::File::create(&doc2_path).unwrap();
        writeln!(doc2, "# Document 2").unwrap();

        let mut analyzer = LinkAnalyzer::new(base_path.to_path_buf());
        analyzer.analyze_directory().unwrap();

        let broken_links = analyzer.find_broken_links();
        assert_eq!(broken_links.len(), 1);
        assert_eq!(broken_links[0].link.text, "Broken link");
        assert_eq!(broken_links[0].link.target, "./nonexistent.md");
    }

    #[test]
    fn test_broken_links_in_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let docs_dir = base_path.join("docs");
        fs::create_dir(&docs_dir).unwrap();

        let doc1_path = base_path.join("README.md");
        let mut doc1 = fs::File::create(&doc1_path).unwrap();
        writeln!(doc1, "[Link to docs](./docs/guide.md)").unwrap();
        writeln!(doc1, "[Broken link](./docs/missing.md)").unwrap();

        let guide_path = docs_dir.join("guide.md");
        let mut guide = fs::File::create(&guide_path).unwrap();
        writeln!(guide, "# Guide").unwrap();

        let mut analyzer = LinkAnalyzer::new(base_path.to_path_buf());
        analyzer.analyze_directory().unwrap();

        let broken_links = analyzer.find_broken_links();
        assert_eq!(broken_links.len(), 1);
        assert_eq!(broken_links[0].link.target, "./docs/missing.md");
    }

    #[test]
    fn test_find_orphaned_documents() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let readme_path = base_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).unwrap();
        writeln!(readme, "[Link to doc1](./doc1.md)").unwrap();

        let doc1_path = base_path.join("doc1.md");
        let mut doc1 = fs::File::create(&doc1_path).unwrap();
        writeln!(doc1, "[Link to doc2](./doc2.md)").unwrap();

        let doc2_path = base_path.join("doc2.md");
        let mut doc2 = fs::File::create(&doc2_path).unwrap();
        writeln!(doc2, "# Doc 2").unwrap();

        let orphaned_path = base_path.join("orphaned.md");
        let mut orphaned = fs::File::create(&orphaned_path).unwrap();
        writeln!(orphaned, "# Orphaned Document").unwrap();

        let mut analyzer = LinkAnalyzer::new(base_path.to_path_buf());
        analyzer.analyze_directory().unwrap();

        let orphaned_docs = analyzer.find_orphaned_documents();
        assert_eq!(orphaned_docs.len(), 1);
        assert!(orphaned_docs[0].ends_with("orphaned.md"));
    }

    #[test]
    fn test_readme_not_orphaned() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let readme_path = base_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).unwrap();
        writeln!(readme, "# Main README").unwrap();

        let mut analyzer = LinkAnalyzer::new(base_path.to_path_buf());
        analyzer.analyze_directory().unwrap();

        let orphaned_docs = analyzer.find_orphaned_documents();
        assert_eq!(orphaned_docs.len(), 0);
    }

    #[test]
    fn test_get_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let doc1_path = base_path.join("doc1.md");
        let mut doc1 = fs::File::create(&doc1_path).unwrap();
        writeln!(doc1, "[Internal link](./doc2.md)").unwrap();
        writeln!(doc1, "[External link](https://example.com)").unwrap();

        let doc2_path = base_path.join("doc2.md");
        let mut doc2 = fs::File::create(&doc2_path).unwrap();
        writeln!(doc2, "[Another internal](./doc1.md)").unwrap();
        writeln!(doc2, "[Broken link](./missing.md)").unwrap();

        let orphaned_path = base_path.join("orphaned.md");
        let mut orphaned = fs::File::create(&orphaned_path).unwrap();
        writeln!(orphaned, "# Orphaned").unwrap();

        let mut analyzer = LinkAnalyzer::new(base_path.to_path_buf());
        analyzer.analyze_directory().unwrap();

        let stats = analyzer.get_statistics();

        assert_eq!(stats.total_documents, 3);
        assert_eq!(stats.total_links, 4);
        assert_eq!(stats.internal_links, 3);
        assert_eq!(stats.external_links, 1);
        assert_eq!(stats.broken_links, 1);
        assert_eq!(stats.orphaned_documents, 1);

        assert_eq!(stats.document_stats.len(), 3);
    }
}
