use std::path::PathBuf;

// It's test code, there's probably a better way to do this but im tired
#[allow(dead_code)]
pub const TEST_QUERY: &str = "twitter";
#[allow(dead_code)]
pub const TEST_ANALYSIS_STORAGE_LOCATION: &str = "test_analysis";

#[allow(dead_code)]
pub(crate) async fn get_test_response() -> egg_mode::Response<egg_mode::search::SearchResult> {
  let token = crate::auth::get_token(std::path::Path::new("auth/bearer.token")).unwrap();
  egg_mode::search::search(TEST_QUERY)
    .result_type(egg_mode::search::ResultType::Recent)
    .count(1)
    .call(&token)
    .await
    .unwrap()
}

#[allow(dead_code)]
pub(crate) fn get_test_date() -> chrono::DateTime<chrono::Utc> {
  chrono::Utc::now()
}

#[allow(dead_code)]
pub(crate) fn clean_test_area() {
  println!(
    "Cleaning test analysis storage area at {:?}",
    PathBuf::from(TEST_ANALYSIS_STORAGE_LOCATION).canonicalize()
  );
  std::fs::remove_dir_all(&TEST_ANALYSIS_STORAGE_LOCATION)
    .expect("Could not clean out storage area!");
}
