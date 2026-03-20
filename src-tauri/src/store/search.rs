use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument, TantivyError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("tantivy error: {0}")]
    Tantivy(#[from] TantivyError),
    #[error("query parse error: {0}")]
    Query(#[from] tantivy::query::QueryParserError),
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub uid: i64,
    pub folder: String,
    pub score: f32,
}

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    schema: Schema,
    uid_field: Field,
    folder_field: Field,
    subject_field: Field,
    from_field: Field,
    to_field: Field,
    body_field: Field,
}

impl SearchIndex {
    pub fn open(path: &Path) -> Result<Self, SearchError> {
        let mut schema_builder = Schema::builder();
        let uid_field = schema_builder.add_i64_field("uid", INDEXED | STORED);
        let folder_field = schema_builder.add_text_field("folder", STRING | STORED);
        let subject_field = schema_builder.add_text_field("subject", TEXT | STORED);
        let from_field = schema_builder.add_text_field("from", TEXT | STORED);
        let to_field = schema_builder.add_text_field("to", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();

        std::fs::create_dir_all(path).ok();
        let dir = MmapDirectory::open(path).map_err(|e| {
            TantivyError::SystemError(format!("failed to open directory: {e}"))
        })?;
        let index = Index::open_or_create(dir, schema.clone())?;
        let reader = index.reader()?;

        Ok(Self {
            index,
            reader,
            schema,
            uid_field,
            folder_field,
            subject_field,
            from_field,
            to_field,
            body_field,
        })
    }

    pub fn writer(&self) -> Result<IndexWriter, SearchError> {
        let writer = self.index.writer(50_000_000)?;
        Ok(writer)
    }

    pub fn index_message(
        &self,
        writer: &IndexWriter,
        uid: i64,
        folder: &str,
        subject: &str,
        from: &str,
        to: &str,
        body: &str,
    ) -> Result<(), SearchError> {
        writer.add_document(doc!(
            self.uid_field => uid,
            self.folder_field => folder,
            self.subject_field => subject,
            self.from_field => from,
            self.to_field => to,
            self.body_field => body,
        ))?;
        Ok(())
    }

    pub fn commit(&self, mut writer: IndexWriter) -> Result<(), SearchError> {
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(
        &self,
        query_str: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.subject_field,
                self.from_field,
                self.to_field,
                self.body_field,
            ],
        );
        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let retrieved: TantivyDocument = searcher.doc(doc_address)?;
            let uid = retrieved
                .get_first(self.uid_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let folder = retrieved
                .get_first(self.folder_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            results.push(SearchResult { uid, folder, score });
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_index_and_search_body() {
        let dir = tempdir().unwrap();
        let search_index = SearchIndex::open(dir.path()).unwrap();
        let writer = search_index.writer().unwrap();

        search_index
            .index_message(
                &writer,
                1,
                "INBOX",
                "Hello World",
                "alice@example.com",
                "bob@example.com",
                "This is a test message about tantivy search.",
            )
            .unwrap();

        search_index
            .index_message(
                &writer,
                2,
                "INBOX",
                "Meeting Tomorrow",
                "bob@example.com",
                "alice@example.com",
                "Let us discuss the project details at noon.",
            )
            .unwrap();

        search_index.commit(writer).unwrap();

        let results = search_index.search("tantivy", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid, 1);
        assert_eq!(results[0].folder, "INBOX");
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_search_by_from_field() {
        let dir = tempdir().unwrap();
        let search_index = SearchIndex::open(dir.path()).unwrap();
        let writer = search_index.writer().unwrap();

        search_index
            .index_message(
                &writer,
                1,
                "INBOX",
                "Hello",
                "alice@example.com",
                "bob@example.com",
                "First message body.",
            )
            .unwrap();

        search_index
            .index_message(
                &writer,
                2,
                "Sent",
                "Reply",
                "bob@example.com",
                "alice@example.com",
                "Second message body.",
            )
            .unwrap();

        search_index.commit(writer).unwrap();

        let results = search_index.search("from:alice", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid, 1);
        assert_eq!(results[0].folder, "INBOX");
    }
}
