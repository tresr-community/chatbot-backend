// TODO: Switch to native workers-rs API when Vectorize support added.
// Issue: https://github.com/cloudflare/workers-rs/issues/709 (open as of Feb 2025)
//
// For now, use HTTP to Workers AI / Vectorize REST APIs.
// - Workers AI: https://developers.cloudflare.com/workers-ai/models/#embeddings
// - Vectorize: https://developers.cloudflare.com/vectorize/reference/client-api/

use nanohtml2text::html2text;
use serde_json::{json, Value};
use worker::*;

const DOCS_WEBSITE: &str = "https://docs.nftreasure.com";
// Model dimensions for @cf/baai/bge-small-en-v1.5 is 384
const EMBEDDING_MODEL: &str = "@cf/baai/bge-small-en-v1.5";

// --- Structures for clearer JSON handling ---
#[derive(serde::Deserialize)]
struct CfApiResponse<T> {
    result: T,
}

#[derive(serde::Deserialize)]
struct EmbeddingResult {
    data: Vec<Vec<f32>>,
}

#[derive(serde::Deserialize)]
struct VectorizeQueryResponse {
    matches: Vec<VectorizeMatch>,
}

#[derive(serde::Deserialize)]
struct VectorizeMatch {
    metadata: Option<Value>,
}

const DOC_SECTIONS_SLUGS: &[&str] = &[
    "quick-start",
    "overview",
    "manifesto",
    "assumptions",
    "nfc-technology",
    "create-merch-and-apparel",
    "mint-a-treasure-key",
    "why-gamefi",
    "tokenomics",
    "token-buybacks",
    "contracts-and-wallets",
    "project-history",
    "governance",
    "team",
    "funding-and-partners",
    "disclaimer",
    "project-y",
    "agency-services",
];

fn doc_sections() -> Vec<(&'static str, String)> {
    DOC_SECTIONS_SLUGS
        .iter()
        .map(|&slug| (slug, format!("{DOCS_WEBSITE}/{slug}")))
        .collect()
}

fn chunk_text(text: String, max_len_words: usize) -> Vec<String> {
    // Basic chunking. Ideally, use a sentence splitter, but this works for now.
    text.split_whitespace()
        .collect::<Vec<_>>()
        .chunks(max_len_words)
        .map(|chunk| chunk.join(" "))
        .filter(|c| c.len() > 50) // Skip very short noise chunks
        .collect()
}

async fn fetch_clean_text(url: &str) -> Result<String> {
    let headers = Headers::new();
    headers.set("User-Agent", "Bot-Indexer/1.0")?; // Polite User Agent

    let mut resp = Fetch::Request(Request::new_with_init(
        url,
        &RequestInit {
            method: Method::Get,
            headers,
            ..Default::default()
        },
    )?)
    .send()
    .await?;

    if resp.status_code() != 200 {
        return Err(Error::from(format!(
            "Failed to fetch {}: {}",
            url,
            resp.status_code()
        )));
    }

    let html = resp.text().await?;
    // Strip HTML tags to index only the content
    Ok(html2text(&html))
}

// Batched Embedding
async fn embed_batch(env: &Env, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let account_id = env.var("CF_ACCOUNT_ID")?.to_string();
    let api_token = env.var("RAG_SECRET")?.to_string();

    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
        account_id, EMBEDDING_MODEL
    );

    // Workers AI accepts an array of strings for batch processing
    let input = json!({ "text": texts });

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Authorization", &format!("Bearer {}", api_token))?;

    let req = Request::new_with_init(
        &url,
        &RequestInit {
            method: Method::Post,
            body: Some(input.to_string().into()),
            headers,
            ..Default::default()
        },
    )?;

    let mut resp = Fetch::Request(req).send().await?;

    if resp.status_code() != 200 {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(Error::from(format!(
            "AI API Error {}: {}",
            resp.status_code(),
            err_text
        )));
    }

    let body: CfApiResponse<EmbeddingResult> = resp.json().await?;

    // Map response back to vector of vectors
    Ok(body.result.data)
}

pub async fn index_docs(env: &Env) -> Result<()> {
    let index_name = "tresr-docs";
    let account_id = env.var("CF_ACCOUNT_ID")?.to_string();
    let vectorize_token = env.var("RAG_SECRET")?.to_string();

    // Accumulate vectors to upsert in batches (Vectorize limit is usually 1000 per call, we do less to be safe)
    let mut vectors_buffer = Vec::new();
    let upsert_url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/vectorize/v2/indexes/{}/upsert",
        account_id, index_name
    );

    for (slug, url) in doc_sections() {
        console_log!("Processing: {}", slug);

        let content = match fetch_clean_text(&url).await {
            Ok(c) => c,
            Err(e) => {
                console_error!("Skipping {}: {}", slug, e);
                continue;
            }
        };

        let chunks = chunk_text(content, 300); // reduced chunk size slightly
        if chunks.is_empty() {
            continue;
        }

        // Get embeddings for all chunks in this page in ONE request
        let embeddings = match embed_batch(env, chunks.clone()).await {
            Ok(e) => e,
            Err(e) => {
                console_error!("Embedding failed for {}: {}", slug, e);
                return Err(e);
            }
        };

        // Prepare Vectorize objects
        for (i, (chunk, emb)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let id = format!("{}-{}", slug, i);

            // Truncate chunk to prevent metadata size limits (Vectorize limit: ~10KB per vector)
            let truncated_chunk = if chunk.len() > 1000 {
                format!("{}...", &chunk[..1000])
            } else {
                chunk.clone()
            };

            vectors_buffer.push(json!({
                "id": id,
                "values": emb,
                "metadata": {
                    "url": url,
                    "section": slug,
                    "text": truncated_chunk,
                }
            }));
        }

        // Upsert if buffer gets large (e.g., every 10 vectors) or end of site
        if vectors_buffer.len() >= 10 {
            perform_upsert(&upsert_url, &vectorize_token, &vectors_buffer).await?;
            vectors_buffer.clear();
        }
    }

    // Final flush
    if !vectors_buffer.is_empty() {
        perform_upsert(&upsert_url, &vectorize_token, &vectors_buffer).await?;
    }

    console_log!("RAG indexing completed successfully");
    Ok(())
}

async fn perform_upsert(url: &str, token: &str, vectors: &[Value]) -> Result<()> {
    console_log!("Upserting {} vectors", vectors.len());
    // Use NDJSON format: one JSON per line
    let ndjson_lines: Vec<String> = vectors
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect();
    let input = ndjson_lines.join("\n");
    let headers = Headers::new();
    headers.set("Content-Type", "application/x-ndjson")?;
    headers.set("Authorization", &format!("Bearer {}", token))?;
    headers.set("CF-API-Version", "2")?;

    let req = Request::new_with_init(
        url,
        &RequestInit {
            method: Method::Post,
            body: Some(input.into()),
            headers,
            ..Default::default()
        },
    )?;

    let mut resp = Fetch::Request(req).send().await?;
    if resp.status_code() != 200 {
        let err = resp.text().await.unwrap_or_default();
        console_error!("Upsert failed: {}", err);
        return Err(Error::from(format!("Upsert Error: {}", err)));
    }
    console_log!("Successfully upserted {} vectors", vectors.len());
    Ok(())
}

pub async fn augment_prompt(env: &Env, message: &str) -> Result<String> {
    // Embed the query (single string, handled as vec of 1)
    let embs = embed_batch(env, vec![message.to_string()]).await?;
    let emb = embs.first().ok_or(Error::from("Embedding failed"))?;

    let index_name = "tresr-docs";
    let account_id = env.var("CF_ACCOUNT_ID")?.to_string();
    let vectorize_token = env.var("RAG_SECRET")?.to_string();

    let query_url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/vectorize/v2/indexes/{}/query",
        account_id, index_name
    );

    let input = json!({
        "vector": emb,
        "top_k": 3,
        "returnMetadata": "true" // Explicitly ask for metadata
    });

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Authorization", &format!("Bearer {}", vectorize_token))?;
    headers.set("CF-API-Version", "2")?;

    let req = Request::new_with_init(
        &query_url,
        &RequestInit {
            method: Method::Post,
            body: Some(input.to_string().into()),
            headers,
            ..Default::default()
        },
    )?;

    let mut resp = Fetch::Request(req).send().await?;

    // FIX: Parse the Cloudflare envelope {"result": ... }
    let body: CfApiResponse<VectorizeQueryResponse> = resp.json().await?;

    let mut context = String::new();
    for match_ in body.result.matches {
        if let Some(meta) = match_.metadata {
            let text = meta.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let url = meta.get("url").and_then(|u| u.as_str()).unwrap_or("");
            context.push_str(&format!("From {}: {}\n\n", url, text));
        }
    }

    if context.is_empty() {
        return Ok("No relevant documentation found.".to_string());
    }

    Ok(context)
}
