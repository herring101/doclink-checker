use clap::{Parser, Subcommand};
use colored::*;
use doclink_checker::{LinkAnalyzer, LinkStatistics};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "doclink-checker")]
#[command(about = "A tool to analyze markdown documents for broken links and statistics")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for broken links in markdown documents
    Check {
        /// Directory to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show statistics about links in markdown documents
    Stats {
        /// Directory to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Find orphaned documents (not linked from anywhere)
    Orphans {
        /// Directory to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, verbose } => {
            if let Err(e) = check_links(path, verbose) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        }
        Commands::Stats { path, format } => {
            if let Err(e) = show_statistics(path, &format) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        }
        Commands::Orphans { path } => {
            if let Err(e) = find_orphans(path) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        }
    }
}

fn check_links(path: PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LinkAnalyzer::new(path.clone());
    analyzer.analyze_directory()?;
    
    let broken_links = analyzer.find_broken_links();
    
    if broken_links.is_empty() {
        println!("{} No broken links found!", "✓".green().bold());
        return Ok(());
    }
    
    println!("{} Found {} broken links:", "✗".red().bold(), broken_links.len());
    
    for broken_link in &broken_links {
        let file_path = broken_link.link.file_path.strip_prefix(&path)
            .unwrap_or(&broken_link.link.file_path);
        
        println!();
        println!("  {} {}:{}", 
                 "File:".yellow().bold(), 
                 file_path.display(),
                 broken_link.link.line_number);
        println!("  {} {}", 
                 "Link:".cyan().bold(), 
                 broken_link.link.text);
        println!("  {} {}", 
                 "Target:".magenta().bold(), 
                 broken_link.link.target);
        println!("  {} {}", 
                 "Reason:".red().bold(), 
                 broken_link.reason);
        
        if verbose {
            println!("  {} [{}]({})", 
                     "Markdown:".blue().bold(),
                     broken_link.link.text,
                     broken_link.link.target);
        }
    }
    
    process::exit(1);
}

fn show_statistics(path: PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LinkAnalyzer::new(path);
    analyzer.analyze_directory()?;
    
    let stats = analyzer.get_statistics();
    
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        "text" | _ => {
            print_text_statistics(&stats);
        }
    }
    
    Ok(())
}

fn print_text_statistics(stats: &LinkStatistics) {
    println!("{}", "Document Link Statistics".bold().underline());
    println!();
    
    println!("{} {}", "Total Documents:".cyan().bold(), stats.total_documents);
    println!("{} {}", "Total Links:".cyan().bold(), stats.total_links);
    println!("{} {} ({}%)", 
             "Internal Links:".green().bold(), 
             stats.internal_links,
             if stats.total_links > 0 { (stats.internal_links * 100) / stats.total_links } else { 0 });
    println!("{} {} ({}%)", 
             "External Links:".blue().bold(), 
             stats.external_links,
             if stats.total_links > 0 { (stats.external_links * 100) / stats.total_links } else { 0 });
    
    if stats.broken_links > 0 {
        println!("{} {}", "Broken Links:".red().bold(), stats.broken_links);
    } else {
        println!("{} {}", "Broken Links:".green().bold(), stats.broken_links);
    }
    
    if stats.orphaned_documents > 0 {
        println!("{} {}", "Orphaned Documents:".yellow().bold(), stats.orphaned_documents);
    } else {
        println!("{} {}", "Orphaned Documents:".green().bold(), stats.orphaned_documents);
    }
    
    if !stats.document_stats.is_empty() {
        println!();
        println!("{}", "Per-Document Statistics:".bold().underline());
        
        for (doc_path, doc_stats) in &stats.document_stats {
            let file_name = doc_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            println!("  {} {} links ({} internal, {} external)",
                     file_name.magenta().bold(),
                     doc_stats.total_links,
                     doc_stats.internal_links,
                     doc_stats.external_links);
        }
    }
}

fn find_orphans(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LinkAnalyzer::new(path.clone());
    analyzer.analyze_directory()?;
    
    let orphaned_docs = analyzer.find_orphaned_documents();
    
    if orphaned_docs.is_empty() {
        println!("{} No orphaned documents found!", "✓".green().bold());
        return Ok(());
    }
    
    println!("{} Found {} orphaned documents:", "⚠".yellow().bold(), orphaned_docs.len());
    
    for orphaned_doc in orphaned_docs {
        let file_path = orphaned_doc.strip_prefix(&path)
            .unwrap_or(&orphaned_doc);
        println!("  {}", file_path.display().to_string().red());
    }
    
    Ok(())
}