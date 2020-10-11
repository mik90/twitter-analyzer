use crate::analysis::SearchAnalysis;
use std::path::PathBuf;

use std::path::Path;

const DEFAULT_STORAGE_LOCATION: &str = "analysis";

pub fn store(item: &SearchAnalysis, location: &Path) {
  let path = item.to_storage_location(location);
  println!("Storing analysis in {:?}", path);
}

#[tokio::test]
async fn test_analysis_storage() {
  crate::test::setup_test_storage();
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

  // There should only be one handle in the base directory
  assert_eq!(
    std::fs::read_dir(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION)
      .unwrap()
      .into_iter()
      .count(),
    1,
    "There should be a single file in {}",
    crate::test::TEST_ANALYSIS_STORAGE_LOCATION,
  );

  let handle_dir: PathBuf = std::fs::read_dir(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION)
    .unwrap()
    .into_iter()
    .next()
    .unwrap()
    .unwrap()
    .path();
  // There should only be one date in the handle's directory
  assert_eq!(
    std::fs::read_dir(&handle_dir).unwrap().into_iter().count(),
    1,
    "There should be a single file in {:?}",
    handle_dir
  );

  let date_dir: PathBuf = std::fs::read_dir(&handle_dir)
    .unwrap()
    .into_iter()
    .next()
    .unwrap()
    .unwrap()
    .path();
  // There should only be one analsyis.json in the date's directory
  assert_eq!(
    std::fs::read_dir(&date_dir).unwrap().into_iter().count(),
    1,
    "There should be a single file in {:?}",
    date_dir
  );

  /* @TODO Check expected filename
   * let expected_filename = Some(std::ffi::OsStr::new("analysis.json"));
   * assert_eq!(actual_filename, expected_filename);
   */
}
