use crate::analysis::SearchAnalysis;

use std::fs;
use std::io::Write;
use std::path::Path;

const DEFAULT_STORAGE_LOCATION: &str = "analysis";

pub fn store(item: &SearchAnalysis) -> Result<(), std::io::Error> {
  store_with_location(item, &Path::new(DEFAULT_STORAGE_LOCATION))
}
/**
 *  Stores a SearchAnalysis and directory structure indicates handle and date
 *  Base location is optional and will default to DEFAULT_STORAGE_LOCATION.
 */
pub fn store_with_location(item: &SearchAnalysis, base_dir: &Path) -> Result<(), std::io::Error> {
  let storage_path = item.storage_location(base_dir);
  println!("Storing analysis as {:?}", &storage_path);
  let parent_dir = storage_path.parent().unwrap();
  if fs::metadata(&parent_dir).is_err() {
    fs::create_dir_all(&parent_dir).expect("Could not create directory despite it not being there");
  }
  let serialized_item = serde_json::to_string(item)?;
  let mut file = std::fs::File::create(&storage_path)?;
  file.write(serialized_item.as_bytes())?;
  Ok(())
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

  store_with_location(
    &analysis,
    &Path::new(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION),
  )
  .expect("Could not store analysis!");

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

  let handle_dir: std::path::PathBuf =
    std::fs::read_dir(&crate::test::TEST_ANALYSIS_STORAGE_LOCATION)
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

  let date_dir: std::path::PathBuf = std::fs::read_dir(&handle_dir)
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
