use crate::analysis::SearchAnalysis;

use crate::twitter::QueryResult;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;

pub const DEFAULT_ANALYSIS_DIR: &str = "analyses";
pub const DEFAULT_QUERY_RESULT_DIR: &str = "queries";
pub const QUERY_RESULT_FILENAME: &str = "query-result.json";

/// Retrieves results over multiple dates from a query
fn retrieve_results_from_query(query_dir: &Path) -> io::Result<Vec<QueryResult>> {
  let mut results = Vec::new();
  if query_dir.is_dir() {
    for date in std::fs::read_dir(query_dir)? {
      let date_dir = date?.path();
      // Should be a date
      if date_dir.is_dir() {
        for result in std::fs::read_dir(date_dir)? {
          let path = result?.path();
          if path.is_file() && path.to_str().unwrap_or("") == QUERY_RESULT_FILENAME {
            // It's a query result so deserialize it!
            let serial_query = std::fs::read(&path)?;
            let deserialized_item: QueryResult = serde_json::from_slice(&serial_query)?;
            results.push(deserialized_item);
          }
        }
        return Ok(results);
      } else {
        let err = format!("{} is not a directory", query_dir.to_str().unwrap_or(""));
        return Err(Error::new(ErrorKind::NotFound, err));
      }
    }
    Err(Error::new(
      ErrorKind::NotFound,
      "Did not find any query results",
    ))
  } else {
    let err = format!("{} is not a directory", query_dir.to_str().unwrap_or(""));
    Err(Error::new(ErrorKind::NotFound, err))
  }
}

/// Retrieve specific queries, (or any) from a given directory
pub fn retrieve_queries(base_dir: &Path, queries: Vec<&str>) -> io::Result<Vec<QueryResult>> {
  if base_dir.is_dir() {
    // Recurse down
    let mut results = Vec::new();
    for entry in std::fs::read_dir(base_dir)? {
      let entry = entry?;
      let path = entry.path();
      // Check if directory is named the same as a query
      // Or just go into it if it's empty
      if path.is_dir() && (queries.contains(&path.to_str().unwrap_or("")) || queries.is_empty()) {
        // The path is a query we're searching for, so recurse down
        results.append(&mut retrieve_results_from_query(&path)?);
      }
    }
    Ok(results)
  } else {
    let err = format!("{:?} is not a directory", base_dir.to_str());
    Err(Error::new(ErrorKind::NotFound, err))
  }
}

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
  println!("Storing query result as {:?}", &storage_path);
  let parent_dir = storage_path.parent().unwrap();
  if fs::metadata(&parent_dir).is_err() {
    fs::create_dir_all(&parent_dir).expect("Could not create directory despite it not being there");
  }
  let serialized_item = serde_json::to_string(query_result)?;
  let mut file = std::fs::File::create(&storage_path)?;
  file.write_all(serialized_item.as_bytes())?;
  Ok(())
}

pub fn clean_storage_area() {
  if Path::new(&DEFAULT_ANALYSIS_DIR).exists() {
    std::fs::remove_dir_all(&DEFAULT_ANALYSIS_DIR)
      .expect("Could not clean out analysis storage area!");
  }
  if Path::new(&DEFAULT_QUERY_RESULT_DIR).exists() {
    std::fs::remove_dir_all(&DEFAULT_QUERY_RESULT_DIR)
      .expect("Could not clean out query storage area!");
  }
}

#[tokio::test]
async fn test_analysis_storage() {
  crate::test::setup_test_dir(Path::new(crate::test::TEST_ANALYSIS_STORAGE_LOCATION));
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
  crate::test::setup_test_dir(Path::new(crate::test::TEST_QUERY_RESULT_STORAGE_LOCATION));
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
