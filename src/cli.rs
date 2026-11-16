use clap::{Parser, ValueEnum};

use crate::{TreeOptions, model::SortBy};

/// Print a visual directory tree.
#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Directory to display
    #[arg(default_value = ".")]
    pub path: String,
    /// Maximum depth to descend (0 prints only the root)
    #[arg(short = 'L', long)]
    pub depth: Option<usize>,
    /// Include entries whose names begin with a dot
    #[arg(short = 'a', long)]
    pub all: bool,
    /// Do not print the number of direct children beside directories
    #[arg(long)]
    pub no_count: bool,
    /// Print only directories
    #[arg(short = 'd', long = "dirs-only")]
    pub dirs_only: bool,
    /// Follow symbolic links while traversing
    #[arg(short = 'l', long)]
    pub follow_links: bool,
    /// Exclude paths whose file name contains this text; may be repeated
    #[arg(short = 'I', long = "ignore", value_name = "PATTERN")]
    pub ignore: Vec<String>,
    /// How siblings are ordered
    #[arg(long, value_enum, default_value_t = SortArg::Name)]
    pub sort: SortArg,
    /// List entries in reverse order
    #[arg(short = 'r', long)]
    pub reverse: bool,
    /// Use ASCII tree characters (useful for plain terminals)
    #[arg(long)]
    pub ascii: bool,
    /// Print a footer with the total directories and files displayed
    #[arg(short = 's', long)]
    pub summary: bool,
    /// Suppress recoverable filesystem warnings
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortArg {
    Name,
    Modified,
    None,
}

impl From<Cli> for TreeOptions {
    fn from(value: Cli) -> Self {
        Self {
            path: value.path,
            max_depth: value.depth,
            show_hidden: value.all,
            show_counts: !value.no_count,
            directories_only: value.dirs_only,
            follow_links: value.follow_links,
            ignored: value.ignore,
            sort_by: match value.sort {
                SortArg::Name => SortBy::Name,
                SortArg::Modified => SortBy::Modified,
                SortArg::None => SortBy::None,
            },
            reverse: value.reverse,
            ascii: value.ascii,
            show_summary: value.summary,
            quiet: value.quiet,
        }
    }
}
