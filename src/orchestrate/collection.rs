//! Document collection management operations

use crate::error::{Error, Result};
use super::types::{DocumentCollection, Document, SearchRequest, SearchResponse};
use super::OrchestrateClient;

impl OrchestrateClient {
    /// List all document collections
    pub async fn list_collections(&self) -> Result<Vec<DocumentCollection>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections", base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list collections: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        if let Ok(collections) = serde_json::from_str::<Vec<DocumentCollection>>(&text) {
            return Ok(collections);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(collections_array) = obj.get("collections").and_then(|c| c.as_array()) {
                let collections: Result<Vec<DocumentCollection>> = collections_array
                    .iter()
                    .map(|coll| {
                        serde_json::from_value::<DocumentCollection>(coll.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return collections;
            }
        }

        Ok(Vec::new())
    }

    /// Get a specific document collection
    pub async fn get_collection(&self, collection_id: &str) -> Result<DocumentCollection> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections/{}", base_url, collection_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to get collection {}: {} - {}",
                collection_id, status, error_text
            )));
        }

        let collection: DocumentCollection = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(collection)
    }

    /// Get a specific document from a collection
    pub async fn get_document(&self, collection_id: &str, document_id: &str) -> Result<Document> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections/{}/documents/{}", base_url, collection_id, document_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to get document {}: {} - {}",
                document_id, status, error_text
            )));
        }

        let document: Document = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(document)
    }

    /// Delete a document from a collection
    pub async fn delete_document(&self, collection_id: &str, document_id: &str) -> Result<()> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections/{}/documents/{}", base_url, collection_id, document_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to delete document {}: {} - {}",
                document_id, status, error_text
            )));
        }

        Ok(())
    }

    /// Search documents in a collection
    pub async fn search_documents(&self, collection_id: &str, request: SearchRequest) -> Result<SearchResponse> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections/{}/search", base_url, collection_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to search documents: {} - {}",
                status, error_text
            )));
        }

        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(search_response)
    }
}
