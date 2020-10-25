use crate::twitter;
use std::{fs, path::Path};

// It's test code, there's probably a better way to do this but im tired
#[allow(dead_code)]
pub const TEST_QUERY: &str = "@twitter";
#[allow(dead_code)]
pub const TEST_ANALYSIS_STORAGE_LOCATION: &str = "test_analyses";
#[allow(dead_code)]
pub const TEST_QUERY_RESULT_STORAGE_LOCATION: &str = "test_queries";
#[allow(dead_code)]
pub const TEST_QUERY_LOCATION: &str = "test_resources/query-result.json";

#[allow(dead_code)]
pub(crate) fn get_test_query_result() -> twitter::QueryResult {
  // It's a query result so deserialize it!
  let serialized =
    std::fs::read(&Path::new(TEST_QUERY_LOCATION)).expect("Could not get test query result");
  let deserialized_result: twitter::QueryResult =
    serde_json::from_slice(&serialized).expect("Could deserialize test query result");
  deserialized_result
}

#[allow(dead_code)]
pub(crate) fn get_test_date() -> chrono::DateTime<chrono::Utc> {
  chrono::Utc::now()
}

/// Ensures that test area is usable
#[allow(dead_code)]
pub fn setup_test_dir(dir: &Path) {
  if dir.exists() {
    println!("Cleaning storage area at {:?}", dir.canonicalize());
    std::fs::remove_dir_all(dir).expect("Could not clean out storage area!");
  }
  fs::create_dir(dir).expect("Could not create storage area!");
}
