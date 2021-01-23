use std::path::Path;
use walkdir::WalkDir;

pub fn clear_directory(dir: &Path) -> Result<(), std::io::Error> {
    // Get all the valid entries in the dir and delete them
    WalkDir::new(&dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| std::fs::remove_dir_all(entry.into_path()))
        .collect()
}

#[cfg(test)]
pub mod test {

    use super::*;
    use crate::twitter;
    use crate::twitter::{QueryResult, Tweet};
    use std::{fs, path::Path, sync::Once};
    pub const TEST_TEMP_DIR: &str = "test_temp";

    impl QueryResult {
        pub fn create_empty() -> QueryResult {
            QueryResult {
                query: "empty_query".to_string(),
                date_utc: chrono::Utc::now(),
                tweets: Vec::new(),
            }
        }
    }

    pub fn get_test_query_result() -> twitter::QueryResult {
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

    #[allow(dead_code)]
    pub fn clear_temp_dir() {
        // Get all the valid entries in the dir and delete them
        let res = clear_directory(&Path::new(TEST_TEMP_DIR));

        assert!(
            res.is_ok(),
            "Error while deleting directory entry: {:?}",
            res.unwrap_err()
        );
    }

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    async fn test_setup() {
        INIT.call_once(|| {
            // Make sure directory exists and is empty`
            let test_temp_dir = Path::new(&TEST_TEMP_DIR);
            if test_temp_dir.exists() {
                clear_temp_dir();
            } else {
                let res = fs::create_dir(test_temp_dir);
                assert!(
                    res.is_ok(),
                    "Error while creating new directory: {:?}",
                    res.unwrap_err()
                );
            }
            assert!(false);
        })
    }
}
