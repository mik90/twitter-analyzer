use crate::{analysis::SearchAnalysis, twitter::QueryResult};
use std::{fs, io, io::Write, path::Path, path::PathBuf};
use walkdir::WalkDir;

pub const DEFAULT_ANALYSIS_DIR: &str = "analyses";
pub const DEFAULT_QUERY_RESULT_DIR: &str = "queries";

pub struct StorageHandler {
    base_dir: PathBuf,
}

impl StorageHandler {
  const QUERY_RESULT_FILENAME: &str = "query-result.json",
  pub fn new(base_dir: PathBuf) -> StorageHandler {
      StorageHandler {
          base_dir,
      }
  }

  /// Retrieve specific queries, (or any) from a given directory
  pub fn retrieve_all_queries(&self) -> io::Result<Vec<QueryResult>> {
      Ok(WalkDir::new(&self.base_dir)
          .into_iter()
          .filter_map(Result::ok)
          // Grab entries that are query.json files
          .filter(|e| e.file_name().eq(QUERY_RESULT_FILENAME))
          .map(|f| QueryResult::deserialize(f.into_path()))
          .filter_map(Result::ok)
          .collect())
  }

  pub fn save_analysis(&self, item: &SearchAnalysis) -> Result<(), std::io::Error> {
    let storage_path = item.storage_location(self.base_dir);
    println!("Storing analysis as {:?}", &storage_path);
    let parent_dir = storage_path.parent().unwrap();
    if fs::metadata(&parent_dir).is_err() {
        fs::create_dir_all(&parent_dir)
            .expect("Could not create directory despite it not being there");
    }
    let serialized_item = serde_json::to_string(item)?;
    let mut file = fs::File::create(&storage_path)?;
    file.write_all(serialized_item.as_bytes())?;
    Ok(())
  }

  pub fn save_query(&self, query_result: &QueryResult) -> Result<(), std::io::Error> {
    let storage_path = query_result.storage_location(&self.base_dir);
    println!("Storing query result as {:?}", &storage_path);
    let parent_dir = storage_path.parent().unwrap();
    if fs::metadata(&parent_dir).is_err() {
        fs::create_dir_all(&parent_dir)
            .expect("Could not create directory despite it not being there");
    }
    let serialized_item = serde_json::to_string(query_result)?;
    let mut file = fs::File::create(&storage_path)?;
    file.write_all(serialized_item.as_bytes())?;
    Ok(())
  }
}

#[cfg(test)]
mod test {

    use super::StorageHandler;
    use crate::{analysis::SearchAnalysis, twitter::QueryResult, util::test};
    use std::path::Path;

    fn get_test_storage_handler() -> StorageHandler {
      StorageHandler::new(&test::TEST_TEMP_DIR)
    }

    #[tokio::test]
    async fn test_analysis_storage() {
        let analysis = SearchAnalysis::create_empty();
        let storage_handler = get_test_storage_handler();
        storage_handler.save_analysis(&analysis).expect("Could not store analysis!");
    }

    #[tokio::test]
    async fn test_query_storage() {
        let query = QueryResult::create_empty();
        let dir = Path::new(&test::TEST_TEMP_DIR);
        save_query_to(&query, dir).expect("Could not store query!");
    }

    #[tokio::test]
    // Store an empty query twice and ensure that two can be deserialized
    async fn test_query_retrieval() {
        let storage_dir = Path::new(&test::TEST_QUERY_RESULT_STORAGE_LOCATION);

        let query = QueryResult::create_empty();

        let res = save_query_to(&query, &storage_dir);
        assert!(res.is_ok(), "Could not store query 1: {}", res.unwrap_err());
        let res = save_query_to(&query, &storage_dir);
        assert!(res.is_ok(), "Could not store query 2: {}", res.unwrap_err());

        let queries = retrieve_all_queries(storage_dir);
        assert!(queries.is_ok(), format!("Error: {:?}", queries.err()));

        // Esnure both queries are equal since they're the same
        let queries = queries.unwrap();
        assert_eq!(queries.len(), 2, "Queries: {:?}", queries);
        assert_eq!(queries[0].query, queries[1].query);
    }
}
