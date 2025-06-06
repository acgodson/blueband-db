//types
use candid::{CandidType, Deserialize};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use serde_json::{from_slice, to_vec};
use std::borrow::Cow;

// =============================================================================
// CORE TYPES
// =============================================================================

pub type CollectionId = String;
pub type DocumentId = String;
pub type ChunkId = String;
pub type VectorId = String;

// =============================================================================
// DOCUMENT TYPES
// =============================================================================

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ContentType {
    PlainText,
    Markdown,
    Html,
    Pdf,
    Other(String),
}

#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub collection_id: CollectionId,
    pub title: String,
    pub content_type: ContentType,
    pub source_url: Option<String>,
    pub timestamp: u64,
    pub total_chunks: u32,
    pub size: u64,
    pub is_embedded: bool,
    pub checksum: String,
}

#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub id: ChunkId,
    pub document_id: DocumentId,
    pub text: String,
    pub position: u32,
    pub char_start: u64,
    pub char_end: u64,
    pub token_count: Option<u32>,
}

// =============================================================================
// VECTOR TYPES
// =============================================================================

#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub id: VectorId,
    pub document_id: DocumentId,
    pub chunk_id: ChunkId,
    pub embedding: Vec<f32>,
    pub norm: f32,
    pub model: String,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VectorMatch {
    pub score: f64,
    pub document_id: DocumentId,
    pub chunk_id: ChunkId,
    pub document_title: Option<String>,
    pub chunk_text: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct QueryResult {
    pub matches: Vec<VectorMatch>,
    pub total_found: u32,
    pub query_time_ms: u64,
}

// =============================================================================
// COLLECTION TYPES
// =============================================================================

#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub genesis_admin: String,
    pub admins: Vec<String>,
    pub settings: CollectionSettings,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CollectionStats {
    pub document_count: u32,
    pub vector_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CollectionWithStats {
    pub collection: Collection,
    pub stats: CollectionStats,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CollectionSettings {
    pub embedding_model: String,
    pub proxy_url: String,
    pub chunk_size: u32,
    pub chunk_overlap: u32,
    pub max_documents: Option<u32>,
    pub auto_embed: bool,
}

// =============================================================================
// REQUEST/RESPONSE TYPES (for canister functions)
// =============================================================================

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CreateCollectionRequest {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub settings: Option<CollectionSettings>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AddDocumentRequest {
    pub collection_id: CollectionId,
    pub title: String,
    pub content: String,
    pub content_type: Option<ContentType>,
    pub source_url: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct QueryRequest {
    pub collection_id: CollectionId,
    pub query_text: String,
    pub limit: Option<u32>,
    pub min_score: Option<f64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SearchRequest {
    pub collection_id: CollectionId,
    pub query: String,
    pub limit: Option<u32>,
    pub min_score: Option<f64>,
    pub filter: Option<String>,
    pub use_approximate: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SearchResponse {
    pub results: Vec<MemorySearchResult>,
}

// =============================================================================
// STABLE STORAGE IMPLEMENTATIONS - BOUNDED
// =============================================================================

impl Storable for DocumentMetadata {
    const BOUND: Bound = Bound::Bounded {
        max_size: 16_384, // 16KB - plenty for titles, URLs, metadata
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_slice(&bytes).unwrap_or_default()
    }
}

impl Storable for SemanticChunk {
    const BOUND: Bound = Bound::Bounded {
        max_size: 32_768, // 32KB
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_slice(&bytes).unwrap_or_default()
    }
}

impl Storable for Vector {
    const BOUND: Bound = Bound::Bounded {
        max_size: 262_144, // 256KB ⚠️
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_slice(&bytes).unwrap_or_default()
    }
}

impl Storable for Collection {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8_192, // 8KB 
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_slice(&bytes).unwrap_or_default()
    }
}

// Create wrapper types for Vec to implement Storable
#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct StringList(pub Vec<String>);

impl StringList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Storable for StringList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 65_536, // 64KB 
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(&self.0).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(from_slice(&bytes).unwrap_or_default())
    }
}

#[derive(CandidType, Default, Clone, Debug, Serialize, Deserialize)]
pub struct ChunkList(pub Vec<SemanticChunk>);

impl ChunkList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Storable for ChunkList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1_048_576, // 1MB - supports large documents with many chunks
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(to_vec(&self.0).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(from_slice(&bytes).unwrap_or_default())
    }
}

// =============================================================================
// DEFAULT IMPLEMENTATIONS
// =============================================================================

impl Default for CollectionSettings {
    fn default() -> Self {
        Self {
            embedding_model: "text-embedding-ada-002".to_string(),
            proxy_url: "https://api.openai.com/v1".to_string(),
            chunk_size: 512,
            chunk_overlap: 64,
            max_documents: None,
            auto_embed: true,
        }
    }
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::PlainText
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

pub fn current_time() -> u64 {
    ic_cdk::api::time()
}

pub fn generate_id(prefix: &str, content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(current_time().to_be_bytes());
    let hash = hasher.finalize();
    format!(
        "{}_{:x}",
        prefix,
        &hash[..8]
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(256).wrapping_add(b as u64))
    )
}

pub fn calculate_vector_norm(embedding: &[f32]) -> f32 {
    embedding.iter().map(|x| x * x).sum::<f32>().sqrt()
}

pub fn validate_collection_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Collection ID must be 1-64 characters".to_string());
    }

    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Collection ID must contain only alphanumeric characters, underscores, and hyphens"
                .to_string(),
        );
    }

    if id.starts_with("__") || id == "admin" || id == "system" {
        return Err("Collection ID uses reserved prefix or keyword".to_string());
    }

    Ok(())
}

pub fn validate_document_content(content: &str) -> Result<(), String> {
    if content.is_empty() {
        return Err("Document content cannot be empty".to_string());
    }

    if content.len() > 10_000_000 {
        // 10MB limit
        return Err("Document content exceeds 10MB limit".to_string());
    }

    Ok(())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MemorySearchResult {
    pub document_id: DocumentId,
    pub chunk_id: ChunkId,
    pub score: f64,
    pub text: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AdminLevel {
    Genesis, // Can manage admins and all operations
    Regular, // Can manage content but not admins
    None,    // No admin privileges
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AdminInfo {
    pub level: AdminLevel,
    pub is_genesis: bool,
    pub can_manage_admins: bool,
    pub can_manage_content: bool,
}

impl AdminInfo {
    pub fn from_level(level: AdminLevel) -> Self {
        match level {
            AdminLevel::Genesis => Self {
                level: AdminLevel::Genesis,
                is_genesis: true,
                can_manage_admins: true,
                can_manage_content: true,
            },
            AdminLevel::Regular => Self {
                level: AdminLevel::Regular,
                is_genesis: false,
                can_manage_admins: false,
                can_manage_content: true,
            },
            AdminLevel::None => Self {
                level: AdminLevel::None,
                is_genesis: false,
                can_manage_admins: false,
                can_manage_content: false,
            },
        }
    }
}

// =============================================================================
// EMBEDDING TYPES
// =============================================================================

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum EmbeddingModel {
    OpenAIAda002,
    OpenAISmall,
    OpenAILarge,
    Custom(String),
}

impl EmbeddingModel {
    pub fn model_name(&self) -> String {
        match self {
            EmbeddingModel::OpenAIAda002 => "text-embedding-ada-002".to_string(),
            EmbeddingModel::OpenAISmall => "text-embedding-3-small".to_string(),
            EmbeddingModel::OpenAILarge => "text-embedding-3-large".to_string(),
            EmbeddingModel::Custom(name) => name.clone(),
        }
    }

    pub fn expected_dimensions(&self) -> Option<usize> {
        match self {
            EmbeddingModel::OpenAIAda002 => Some(1536),
            EmbeddingModel::OpenAISmall => Some(1536),
            EmbeddingModel::OpenAILarge => Some(3072),
            EmbeddingModel::Custom(_) => None,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub model: EmbeddingModel,
    pub proxy_url: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub usage_tokens: Option<u32>,
}

// =============================================================================
// CACHE TYPES
// =============================================================================

/// Cache statistics for monitoring
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_memory_bytes: usize,
    pub max_memory_bytes: usize,
    pub max_entries: usize,
    pub memory_usage_percent: u32,
}

impl CacheStats {
    pub fn memory_mb(&self) -> f64 {
        self.total_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn max_memory_mb(&self) -> f64 {
        self.max_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn is_near_limit(&self) -> bool {
        self.memory_usage_percent > 80
    }
}
