use std::path::Path;
use walkdir::WalkDir;

pub fn clear_directory(dir: &Path) -> Result<(), std::io::Error> {
    // Get all the valid entries in the dir and delete them
    WalkDir::new(&dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| std::fs::remove_dir_all(entry.into_path()))
        .collect()
}

#[cfg(test)]
pub mod test {

    use crate::analysis::{HandlePattern, SearchAnalysis};
    use crate::twitter::{QueryResult, Tweet};
    pub const TEST_TEMP_DIR: &str = "test_temp";

    /// Create a (mildly) valid SearchAnalysis that can be stored
    pub fn get_dummy_search_analysis() -> SearchAnalysis {
        SearchAnalysis {
            queries: vec!["dummy_search_analysis".to_string()],
            date_utc: chrono::Utc::now(),
            word_frequency: vec![("Hello".to_string(), 1)],
            handle_patterns: vec![(HandlePattern::Other, 1)],
        }
    }

    /// Create a (mildly) valid Query result
    pub fn get_dummy_query_result() -> QueryResult {
        QueryResult {
            query: "@twitter".to_string(),
            // Date doesn't actually matter for test content
            date_utc: chrono::Utc::now(),
            tweets: vec![Tweet {
                text: r#"RT @Twitter: hello hello there are multiple words here, some repeated, hello hello…"#.to_string(),
                handle: "fakeHandle".to_string(),
                date_utc: chrono::Utc::now(),
                retweet_count: 47111,
                favorite_count: 1234,
            }],
        }
    }
}
