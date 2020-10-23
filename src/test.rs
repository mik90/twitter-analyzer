use std::fs;
use std::path::Path;

// It's test code, there's probably a better way to do this but im tired
#[allow(dead_code)]
pub const TEST_QUERY: &str = "@twitter";
#[allow(dead_code)]
pub const TEST_ANALYSIS_STORAGE_LOCATION: &str = "test_analyses";
#[allow(dead_code)]
pub const TEST_QUERY_RESULT_STORAGE_LOCATION: &str = "test_queries";

#[allow(dead_code)]
pub(crate) async fn get_test_response() -> egg_mode::Response<egg_mode::search::SearchResult> {
  let token = crate::auth::get_token(std::path::Path::new("auth/bearer.token")).unwrap();
  let res = egg_mode::search::search(TEST_QUERY)
    .result_type(egg_mode::search::ResultType::Recent)
    .count(1)
    .call(&token)
    .await;
  res.expect("Could not get response!")
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
