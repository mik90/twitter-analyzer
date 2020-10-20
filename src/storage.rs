use crate::analysis::SearchAnalysis;

use crate::twitter::QueryResult;
use std::fs;
use std::io::Write;
use std::path::Path;

pub const DEFAULT_ANALYSIS_DIR: &str = "analysis";
pub const DEFAULT_QUERY_RESULT_DIR: &str = "queries";

pub fn store_analysis(item: &SearchAnalysis) -> Result<(), std::io::Error> {
  store_analysis_with_location(item, &Path::new(DEFAULT_ANALYSIS_DIR))
}
/**
 *  Stores a SearchAnalysis and directory structure indicates handle and date
 *  Base location is optional and will default to DEFAULT_STORAGE_LOCATION.
 */
fn store_analysis_with_location(
  item: &SearchAnalysis,
  base_dir: &Path,
) -> Result<(), std::io::Error> {
  let storage_path = item.storage_location(base_dir);
  println!("Storing analysis as {:?}", &storage_path);
  let parent_dir = storage_path.parent().unwrap();
  if fs::metadata(&parent_dir).is_err() {
    fs::create_dir_all(&parent_dir).expect("Could not create directory despite it not being there");
  }
  let serialized_item = serde_json::to_string(item)?;
  let mut file = std::fs::File::create(&storage_path)?;
  file.write_all(serialized_item.as_bytes())?;
  Ok(())
}

pub fn store_query(query_result: &QueryResult) -> Result<(), std::io::Error> {
  store_query_with_location(&query_result, &Path::new(DEFAULT_QUERY_RESULT_DIR))
}

fn store_query_with_location(
  query_result: &QueryResult,
  base_dir: &Path,
) -> Result<(), std::io::Error> {
  let storage_path = query_result.storage_location(base_dir);
  println!("Storing analysis as {:?}", &storage_path);
  let parent_dir = storage_path.parent().unwrap();
  if fs::metadata(&parent_dir).is_err() {
    fs::create_dir_all(&parent_dir).expect("Could not create directory despite it not being there");
  }
  let serialized_item = serde_json::to_string(query_result)?;
  let mut file = std::fs::File::create(&storage_path)?;
  file.write_all(serialized_item.as_bytes())?;
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

  store_analysis_with_location(
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
}

#[tokio::test]
async fn test_query_storage() {
  crate::test::setup_test_storage();
  let response = crate::test::get_test_response().await;
  let query = QueryResult::new(
    crate::test::TEST_QUERY,
    crate::test::get_test_date(),
    &response,
  );

  store_query_with_location(
    &query,
    &Path::new(&crate::test::TEST_QUERY_RESULT_STORAGE_LOCATION),
  )
  .expect("Could not store query!");

  // There should only be one handle in the base directory
  assert_eq!(
    std::fs::read_dir(&crate::test::TEST_QUERY_RESULT_STORAGE_LOCATION)
      .unwrap()
      .into_iter()
      .count(),
    1,
    "There should be a single file in {}",
    crate::test::TEST_QUERY_RESULT_STORAGE_LOCATION,
  );

  let handle_dir: std::path::PathBuf =
    std::fs::read_dir(&crate::test::TEST_QUERY_RESULT_STORAGE_LOCATION)
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
}
