use crate::{analysis::SearchAnalysis, twitter::QueryResult};
use std::{
  fs, io,
  io::{Error, ErrorKind, Write},
  path::Path,
};

pub const DEFAULT_ANALYSIS_DIR: &str = "analyses";
pub const DEFAULT_QUERY_RESULT_DIR: &str = "queries";
pub const QUERY_RESULT_FILENAME: &str = "query-result.json";

/// Retrieves results over multiple dates from a query
fn retrieve_results_from_query(query_dir: &Path) -> io::Result<Vec<QueryResult>> {
  let mut results = Vec::new();
  println!(
    "Retrieving results from {}",
    query_dir.to_str().unwrap_or("Could not unwrap path!")
  );
  if query_dir.is_dir() {
    for date in fs::read_dir(query_dir)? {
      let date_dir = date?.path();
      println!(
        "Entered {}",
        date_dir.to_str().unwrap_or("Could not unwrap date path!")
      );
      // Should be a date
      if date_dir.is_dir() {
        for result in fs::read_dir(date_dir)? {
          let result_path = result?.path();
          println!(
            "Entered result {}",
            result_path
              .to_str()
              .unwrap_or("Could not unwrap result path!")
          );
          if result_path.is_file() && result_path.ends_with(Path::new(QUERY_RESULT_FILENAME)) {
            // It's a query result so deserialize it!
            let serial_query = fs::read(&result_path)?;
            let deserialized_item: QueryResult = serde_json::from_slice(&serial_query)?;
            results.push(deserialized_item);
            println!("Found query result at {:?}", &result_path);
          }
        }
      } else {
        let err = format!("{} is not a directory", query_dir.to_str().unwrap_or(""));
        return Err(Error::new(ErrorKind::NotFound, err));
      }
    }
    println!();
    Ok(results)
  } else {
    let err = format!("{} is not a directory", query_dir.to_str().unwrap_or(""));
    Err(Error::new(ErrorKind::NotFound, err))
  }
}

/// Retrieve specific queries, (or any) from a given directory
pub fn retrieve_queries(base_dir: &Path, queries: &[&str]) -> io::Result<Vec<QueryResult>> {
  if base_dir.is_dir() {
    // Recurse down
    let mut results = Vec::new();
    for query in fs::read_dir(base_dir)? {
      let query = query?;
      let path = query.path();
      // Check if directory is named the same as a query
      // Or just go into it if we don't have any specific queries
      let path_contains_query = path.is_dir() && queries.contains(&path.to_str().unwrap_or(""));
      if path_contains_query || queries.is_empty() {
        // The path is a query we're searching for, so recurse down
        results.append(&mut retrieve_results_from_query(&path)?);
      }
    }
    if results.is_empty() {
      return Err(Error::new(
        ErrorKind::NotFound,
        "Could not find any query results!",
      ));
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
}
