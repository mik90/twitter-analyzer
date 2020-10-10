use crate::analysis::SearchAnalysis;

use std::path::Path;

const DEFAULT_STORAGE_LOCATION: &str = "analysis";

pub fn store(item: &SearchAnalysis, location: &Path) {
  let path = item.to_storage_location(location);
  println!("Storing analysis in {:?}", path);
}

#[tokio::test]
async fn test_analysis_storage() {
  crate::test::clean_test_area();
  let response = crate::test::get_test_response().await;
  let analysis = SearchAnalysis::new(
    crate::test::TEST_QUERY,
    crate::test::get_test_date(),
    &response,
  )
  .unwrap();
  store(
    &analysis,
    &Path::new(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION),
  );

  let base_dir = std::fs::read_dir(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION).unwrap();
  // There should only be one handle in the base directory
  assert_eq!(base_dir.into_iter().count(), 1);

  let handle_path = base_dir.into_iter().next().unwrap().unwrap().path();
  let handle_dir = std::fs::read_dir(&handle_path).unwrap();
  // There should only be one date in the handle's directory
  assert_eq!(handle_dir.into_iter().count(), 1);

  let date_path = handle_dir.into_iter().next().unwrap().unwrap().path();
  let date_dir = std::fs::read_dir(&date_path).unwrap();
  // There should only be one analsyis.json in the date's directory
  assert_eq!(date_dir.into_iter().count(), 1);

  /* @TODO Check expected filename
   * let expected_filename = Some(std::ffi::OsStr::new("analysis.json"));
   * assert_eq!(actual_filename, expected_filename);
   */
}
