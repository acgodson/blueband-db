// compute/embeddings.rs

use super::{calculate_norm, validate_embedding};
use crate::types::*;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext, TransformFunc,
};
use ic_cdk_macros::query;

/// Calculate cycles needed for HTTP outcall based on embedding model and request
fn calculate_embedding_cycles(
    model: &EmbeddingModel,
    text_count: usize,
    request_body_size: usize,
) -> u128 {
    let n = 13u128;

    let base_fee = (3_000_000 + 60_000 * n) * n;
    let request_fee = 400 * n * (request_body_size as u128);

    // Estimate response size based on embedding dimensions
    let dimensions = model.expected_dimensions().unwrap_or(1536);
    let response_bytes = (dimensions * 4 * text_count) + (text_count * 150) + 1000; // JSON overhead
    let response_fee = 800 * n * (response_bytes as u128);

    let total_calculated = base_fee + request_fee + response_fee;

    // Use 2.5x buffer (learned from XRC and other production canisters)
    let with_buffer = (total_calculated as f64 * 2.5) as u128;

    // Ensure minimum viable amount (1B cycles like XRC)
    with_buffer.max(1_000_000_000)
}

/// Calculate max response bytes for the request
fn calculate_max_response_bytes(model: &EmbeddingModel, text_count: usize) -> u64 {
    let dimensions = match model {
        EmbeddingModel::OpenAIAda002 => 1536,
        EmbeddingModel::OpenAISmall => 1536,
        EmbeddingModel::OpenAILarge => 3072,
        EmbeddingModel::Custom(_) => 1536,
    };

    // Conservative estimate with 50% buffer
    let estimated_bytes = (dimensions * 4 * text_count) + (text_count * 150) + 1000;
    let with_buffer = (estimated_bytes as f64 * 1.5) as u64;

    // Ensure reasonable bounds
    with_buffer.max(50_000).min(2_000_000) // 50KB min, 2MB max
}

/// Generate embeddings for multiple texts via HTTP outcall (SIMPLIFIED)
pub async fn get_embeddings_async(request: EmbeddingRequest) -> Result<EmbeddingResponse, String> {
    validate_embedding_request(&request)?;

    let request_body = create_embedding_request_body(&request)?;
    let request_body_bytes = request_body.as_bytes();

    let idempotency_key = generate_idempotency_key(&request);

    // Calculate cycles using proven ICP formula
    let cycles_needed = calculate_embedding_cycles(
        &request.model,
        request.texts.len(),
        request_body_bytes.len(),
    );

    let max_response_bytes = calculate_max_response_bytes(&request.model, request.texts.len());

    let http_request_arg = CanisterHttpRequestArgument {
        url: request.proxy_url.clone(),
        method: HttpMethod::POST,
        body: Some(request_body_bytes.to_vec()),
        max_response_bytes: Some(max_response_bytes),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform_embedding_response".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "User-Agent".to_string(),
                value: "IC-VectorDB/1.0".to_string(),
            },
            // ADD: Idempotency key for preventing duplicate requests
            HttpHeader {
                name: "Idempotency-Key".to_string(),
                value: idempotency_key,
            },
        ],
    };

    match http_request(http_request_arg, cycles_needed).await {
        Ok((response,)) => {
            if response.status < 200u16 || response.status >= 300u16 {
                return Err(format!(
                    "HTTP {} error: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                        .chars()
                        .take(200)
                        .collect::<String>()
                ));
            }

            parse_embedding_response(&response.body, &request.model)
        }
        Err((rejection_code, message)) => {
            if message.contains("cycles") || message.contains("OutOfCycles") {
                Err(format!(
                    "Insufficient cycles: sent {} cycles but need more. Error: {}",
                    cycles_needed, message
                ))
            } else if message.contains("SysTransient") || message.contains("timeout") {
                Err(format!(
                    "Network error (consider retry): {:?} - {}",
                    rejection_code, message
                ))
            } else {
                Err(format!(
                    "HTTP request failed: {:?} - {}",
                    rejection_code, message
                ))
            }
        }
    }
}

/// Generate embeddings for document chunks
pub async fn embed_document_chunks(
    chunks: &[SemanticChunk],
    collection_settings: &CollectionSettings,
    proxy_url: String,
) -> Result<Vec<Vector>, String> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    // Process chunks in smaller batches to stay within HTTP limits
    const BATCH_SIZE: usize = 3; // Reduced from 5 to 3 to stay well under 50KB
    let mut all_vectors = Vec::new();
    let model = parse_embedding_model(&collection_settings.embedding_model)?;

    for chunk_batch in chunks.chunks(BATCH_SIZE) {
        let texts: Vec<String> = chunk_batch.iter().map(|chunk| chunk.text.clone()).collect();
        
        let request = EmbeddingRequest {
            texts,
            model: model.clone(),
            proxy_url: proxy_url.clone(),
        };

        // Validate request body size before sending
        let request_body = create_embedding_request_body(&request)?;
        if request_body.len() > 45_000 { // Leave some buffer for headers
            return Err("Request body too large, try reducing batch size or chunk size".to_string());
        }

        let response = get_embeddings_async(request).await?;

        if response.embeddings.len() != chunk_batch.len() {
            return Err(format!(
                "Embedding count mismatch: expected {}, got {}",
                chunk_batch.len(),
                response.embeddings.len()
            ));
        }

        for (chunk, embedding) in chunk_batch.iter().zip(response.embeddings.iter()) {
            validate_embedding(embedding)?;

            let norm = calculate_norm(embedding)?;

            let vector = Vector {
                id: generate_id("vec", &chunk.id),
                document_id: chunk.document_id.clone(),
                chunk_id: chunk.id.clone(),
                embedding: embedding.clone(),
                norm,
                model: response.model.clone(),
                created_at: current_time(),
            };

            ic_cdk::println!(
                "Created vector - ID: {}, Document: {}, Chunk: {}, Dimensions: {}, Norm: {:.4}, Model: {}",
                vector.id,
                vector.document_id,
                vector.chunk_id,
                vector.embedding.len(),
                vector.norm,
                vector.model
            );

            all_vectors.push(vector);
        }
    }

    Ok(all_vectors)
}

pub async fn embed_query_text(
    query: &str,
    model: EmbeddingModel,
    proxy_url: String,
) -> Result<(Vec<f32>, f32), String> {
    if query.trim().is_empty() {
        return Err("Query text cannot be empty".to_string());
    }

    let request = EmbeddingRequest {
        texts: vec![query.to_string()],
        model,
        proxy_url,
    };

    let response = get_embeddings_async(request).await?;

    if response.embeddings.is_empty() {
        return Err("No embedding returned for query".to_string());
    }

    let embedding = &response.embeddings[0];
    let norm = calculate_norm(embedding)?;

    Ok((embedding.clone(), norm))
}

fn validate_embedding_request(request: &EmbeddingRequest) -> Result<(), String> {
    if request.texts.is_empty() {
        return Err("No texts provided for embedding".to_string());
    }

    if request.texts.len() > 50 {
        return Err("Too many texts in single request (max 50)".to_string());
    }

    let total_chars: usize = request.texts.iter().map(|t| t.len()).sum();
    if total_chars > 50_000 {
        return Err("Total text content too large (max 50k characters)".to_string());
    }

    if !request.proxy_url.starts_with("https://") {
        return Err("Proxy URL must use HTTPS".to_string());
    }

    for (i, text) in request.texts.iter().enumerate() {
        if text.trim().is_empty() {
            return Err(format!("Text at index {} is empty", i));
        }

        if text.len() > 4000 {
            return Err(format!(
                "Text at index {} is too long ({} chars, max 4000)",
                i,
                text.len()
            ));
        }
    }

    Ok(())
}

fn create_embedding_request_body(request: &EmbeddingRequest) -> Result<String, String> {
    let request_json = serde_json::json!({
        "input": request.texts,
        "model": request.model.model_name(),
        "encoding_format": "float"
    });

    serde_json::to_string(&request_json).map_err(|e| format!("Failed to serialize request: {}", e))
}

fn parse_embedding_response(
    response_body: &[u8],
    model: &EmbeddingModel,
) -> Result<EmbeddingResponse, String> {
    if response_body.len() > 2_000_000 {
        return Err("Response body too large".to_string());
    }

    let body_str = String::from_utf8(response_body.to_vec())
        .map_err(|e| format!("Failed to decode response as UTF-8: {}", e))?;

    if body_str.trim().is_empty() {
        return Err("Received empty response body".to_string());
    }

    let response_json: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    if let Some(error) = response_json.get("error") {
        return Err(format!(
            "API error: {}",
            error.to_string().chars().take(200).collect::<String>()
        ));
    }

    let data = response_json["data"]
        .as_array()
        .ok_or("Missing 'data' field in response")?;

    let mut embeddings = Vec::new();

    for (i, item) in data.iter().enumerate() {
        let embedding_array = item["embedding"]
            .as_array()
            .ok_or_else(|| format!("Missing embedding for item {}", i))?;

        let embedding: Result<Vec<f32>, _> = embedding_array
            .iter()
            .enumerate()
            .map(|(j, v)| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| format!("Invalid float at position {} in embedding {}", j, i))
            })
            .collect();

        let embedding = embedding?;

        if let Some(expected_dim) = model.expected_dimensions() {
            if embedding.len() != expected_dim {
                return Err(format!(
                    "Dimension mismatch for embedding {}: expected {}, got {}",
                    i,
                    expected_dim,
                    embedding.len()
                ));
            }
        }

        validate_embedding(&embedding)?;

        embeddings.push(embedding);
    }

    let usage_tokens = response_json
        .get("usage")
        .and_then(|u| u.get("total_tokens"))
        .and_then(|t| t.as_u64())
        .map(|t| t as u32);

    Ok(EmbeddingResponse {
        embeddings,
        model: model.model_name().to_string(),
        usage_tokens,
    })
}

fn parse_embedding_model(model_str: &str) -> Result<EmbeddingModel, String> {
    match model_str {
        "text-embedding-ada-002" => Ok(EmbeddingModel::OpenAIAda002),
        "text-embedding-3-small" => Ok(EmbeddingModel::OpenAISmall),
        "text-embedding-3-large" => Ok(EmbeddingModel::OpenAILarge),
        custom => Ok(EmbeddingModel::Custom(custom.to_string())),
    }
}

#[query]
fn transform_embedding_response(raw: TransformArgs) -> HttpResponse {
    let mut response = raw.response;

    response.headers.retain(|header| {
        let name_lower = header.name.to_lowercase();

        !name_lower.contains("date")
            && !name_lower.contains("server")
            && !name_lower.contains("x-request-id")
            && !name_lower.contains("x-ratelimit")
            && !name_lower.contains("cf-")
            && !name_lower.contains("set-cookie")
            && name_lower != "age"
            && name_lower != "vary"
    });

    response
}

fn generate_idempotency_key(request: &EmbeddingRequest) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    // Hash ONLY the request content
    hasher.update(request.model.model_name().as_bytes());
    hasher.update(request.proxy_url.as_bytes());

    // Hash each text in deterministic order
    for text in &request.texts {
        hasher.update(text.as_bytes());
        hasher.update(b"|");
    }

    let hash = hasher.finalize();
    format!(
        "icp-vdb-{:x}",
        &hash[..8]
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(256).wrapping_add(b as u64))
    )
}
