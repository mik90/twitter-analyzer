use crate::{analysis::SearchAnalysis, twitter::QueryResult};
use std::{fs, io, io::Write, path::Path, path::PathBuf};
use walkdir::WalkDir;

pub const DEFAULT_STORAGE_DIR: &str = "data";

pub struct StorageHandler {
    base_dir: PathBuf,
}

enum StorageItem {
    Query(QueryResult),
    Analysis(SearchAnalysis),
}

impl StorageHandler {
    const QUERY_RESULT_FILENAME: &'static str = "query-result.json";
    const ANALYSIS_RESULT_FILENAME: &'static str = "analysis-result.json";

    pub fn new() -> StorageHandler {
        let base_dir = PathBuf::from(DEFAULT_STORAGE_DIR);
        if !base_dir.exists() {
            fs::create_dir(&base_dir)
                .expect("Could not create directory despite it not being there");
        }
        StorageHandler {
            base_dir: PathBuf::from(DEFAULT_STORAGE_DIR),
        }
    }

    /// Using a &mut return didn't really mesh well with my usage
    /// Refernce: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#consuming-builders
    pub fn storage_dir(mut self, dir: &Path) -> StorageHandler {
        self.base_dir = PathBuf::from(dir);
        if !self.base_dir.exists() {
            fs::create_dir(&self.base_dir)
                .expect("Could not create directory despite it not being there");
        }
        self
    }

    /// Retrieve specific queries, (or any) from a given directory
    pub fn retrieve_all_queries(&self) -> io::Result<Vec<QueryResult>> {
        Ok(WalkDir::new(&self.base_dir)
            .into_iter()
            // Filter in results that are not errors
            .filter_map(Result::ok)
            // Filter in entries that are .*query.json files
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .ends_with(Self::QUERY_RESULT_FILENAME)
            })
            // Turn the entries into paths
            .map(|entry| QueryResult::deserialize(entry.into_path()))
            // Filter out errors
            .filter_map(Result::ok)
            .collect())
    }

    pub fn save_analysis(&self, item: &SearchAnalysis) -> Result<(), std::io::Error> {
        let storage_path = self.create_storage_path(item);
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

    /// Uses ISO 8601 / RFC 3339 date & time format
    fn create_storage_path(&self, item: StorageItem) -> PathBuf {
        // Adjust filename based on type
        let (query_dir, filename) = match item {
            StorageItem::Analysis(item) => (
                // Foldername will be `query1.query2.query3` etc
                PathBuf::from(&item.queries.join(".")),
                PathBuf::from(format!(
                    "{}.{}",
                    &item.date_utc.format("%+").to_string(),
                    Self::ANALYSIS_RESULT_FILENAME
                )),
            ),
            StorageItem::Query(item) => (
                PathBuf::from(&item.query),
                PathBuf::from(format!(
                    "{}.{}",
                    &item.date_utc.format("%+").to_string(),
                    Self::QUERY_RESULT_FILENAME
                )),
            ),
        };
        let storage_path: PathBuf = [&self.base_dir, &query_dir, &filename].iter().collect();
        storage_path
    }

    pub fn save_query(&self, query_result: &QueryResult) -> Result<(), std::io::Error> {
        let storage_path = self.create_storage_path(StorageItem::Query(query_result));

        println!("Storing query result as {:?}", &storage_path);
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
        StorageHandler::new().storage_dir(&Path::new(&test::TEST_TEMP_DIR))
    }

    #[tokio::test]
    async fn test_analysis_storage() {
        let analysis = SearchAnalysis::create_empty();
        let storage_handler = get_test_storage_handler();
        storage_handler
            .save_analysis(&analysis)
            .expect("Could not store analysis!");
    }

    #[tokio::test]
    async fn test_query_storage() {
        let query = QueryResult::create_empty();
        let storage_handler = get_test_storage_handler();
        storage_handler
            .save_query(&query)
            .expect("Could not store query!");
    }

    #[tokio::test]
    // Store an empty query twice and ensure that two can be deserialized
    async fn test_query_retrieval() {
        let storage_handler = get_test_storage_handler();

        let res = storage_handler.save_query(&QueryResult::create_empty());
        assert!(res.is_ok(), "Could not store query 1: {}", res.unwrap_err());

        let res = storage_handler.save_query(&QueryResult::create_empty());
        assert!(res.is_ok(), "Could not store query 2: {}", res.unwrap_err());

        let queries = storage_handler.retrieve_all_queries();
        assert!(queries.is_ok(), format!("Error: {:?}", queries.err()));

        let queries = queries.unwrap();
        println!("Retrieved queries: {:?}", queries);
        assert_eq!(
            queries.is_empty(),
            false,
            "Expected some queries to be retrieved!"
        );
    }
}
