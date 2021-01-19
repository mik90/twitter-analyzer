#[cfg(test)]
pub mod test {

    use super::*;
    use crate::twitter;
    use crate::twitter::QueryResult;
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
        // It's a query result so deserialize it!
        let serialized = std::fs::read(&Path::new(test::TEST_TEMP_DIR))
            .expect("Could not get test query result");
        let deserialized_result: twitter::QueryResult =
            serde_json::from_slice(&serialized).expect("Could deserialize test query result");
        deserialized_result
    }

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    // Setup logic for unit tests, runs once at start
    #[allow(dead_code)]
    async fn test_setup() {
        INIT.call_once(|| {
            // Make sure directory exists and is empty`
            let test_temp_dir = Path::new(&TEST_TEMP_DIR);
            if test_temp_dir.exists() {
                assert!(test_temp_dir.is_dir());
                println!(
                    "Cleaning storage area at {:?}",
                    test_temp_dir.canonicalize()
                );
                let res = std::fs::remove_dir_all(test_temp_dir);
                assert!(
                    res.is_ok(),
                    "Error while deleting directory: {:?}",
                    res.unwrap_err()
                );
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
