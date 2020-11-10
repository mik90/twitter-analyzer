use crate::{analysis::SearchAnalysis, twitter::QueryResult};
use std::{fs, io, io::Write, path::Path};
use walkdir::WalkDir;

pub const DEFAULT_ANALYSIS_DIR: &str = "analyses";
pub const DEFAULT_QUERY_RESULT_DIR: &str = "queries";
pub const QUERY_RESULT_FILENAME: &str = "query-result.json";

/// Retrieve specific queries, (or any) from a given directory
pub fn retrieve_all_queries(base_dir: &Path) -> io::Result<Vec<QueryResult>> {
  Ok(
    WalkDir::new(base_dir)
      .into_iter()
      .filter_map(Result::ok)
      // Grab entries that are query.json files
      .filter(|e| e.file_name().eq(QUERY_RESULT_FILENAME))
      .map(|f| QueryResult::deserialize(f.into_path()))
      .filter_map(Result::ok)
      .collect(),
  )
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
  let mut file = fs::File::create(&storage_path)?;
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
  let mut file = fs::File::create(&storage_path)?;
  file.write_all(serialized_item.as_bytes())?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::analysis::SearchAnalysis;
  use crate::twitter::test;
  use std::path::Path;

  #[tokio::test]
  async fn test_analysis_storage() {
    let storage_dir = Path::new(&test::TEST_ANALYSIS_STORAGE_LOCATION);
    test::setup_test_dir(&storage_dir);
    let analysis = SearchAnalysis::create_empty();

    store_analysis_with_location(&analysis, &storage_dir).expect("Could not store analysis!");

    assert!(storage_dir.exists());
  }

  #[tokio::test]
  async fn test_query_storage() {
    let storage_dir = Path::new(&test::TEST_QUERY_RESULT_STORAGE_LOCATION);
    test::setup_test_dir(&storage_dir);
    let query = QueryResult::create_empty();

    store_query_with_location(&query, &storage_dir).expect("Could not store query!");

    assert!(storage_dir.exists());
  }

  #[tokio::test]
  async fn test_query_retrieval() {
    let storage_dir = Path::new(&test::TEST_QUERY_RESULT_STORAGE_LOCATION);
    test::setup_test_dir(&storage_dir);

    let query = QueryResult::create_empty();
    let res = store_query_with_location(&query, &storage_dir);
    assert!(res.is_ok(), "Could not store query: {}", res.unwrap_err());

    assert!(storage_dir.exists());

    let query = QueryResult::create_empty();
    let res = store_query_with_location(&query, &storage_dir);
    assert!(res.is_ok(), "Could not store query: {}", res.unwrap_err());

    let queries = retrieve_all_queries(storage_dir);
    assert!(queries.is_ok(), format!("Error: {:?}", queries.err()));
    let queries = queries.unwrap();
    assert_eq!(queries.len(), 2, "Queries: {:?}", queries);
    assert_eq!(queries[0].query, queries[1].query);
  }
}
