# Prompt

The _Ron Jay_ AI prompt.

## Overview

The prompt is set statically inside the `workers/backend/src/ai/cloudflare/v1/prompt.rs` file.

In order to make Ron Jay an expert in the TRESR documentation, we use a RAG.

## RAG

RAG stands for Retrieval Augmented Generation. It is a technique that combines the power of large language models with the ability to retrieve relevant information from a knowledge base.

### Creation

This is how it was created.

- Create the vector database

```bash
wrangler vectorize create tresr-docs --dimensions=384 --metric=cosine
```

- Ensure the `wrangler.toml` file has the following configuration for _all environments_.

```toml
[[ai]]
binding = "AI"

[[vectorize]]
binding = "TRESR_DOCS"
index_name = "tresr-docs"
remote = true # This is needed for local development if you intend to index locally.
```

- Start the worker locally using the just command

```bash
just start
```

- Index the docs using the admin URL for the environment.

```bash
export RAG_SECRET=<RAG Secret goes here>
export RAG_URL="http://localhost:9200/ai/admin/rag-index"                                        # Development
#export RAG_URL="https://preview.chat.tresr.community/ai/admin/rag-index"              # Preview
#export RAG_URL="https://chat.tresr.community/ai/admin/rag-index"                         # Production

curl \
  --request POST \
  --url ${RAG_URL} \
  --header "X-Admin-Secret: ${RAG_SECRET}"
```

### Updating

This is how you can update the index.

- Delete the Vector Database

```bash
wrangler vectorize delete tresr-docs
wrangler vectorize create tresr-docs --dimensions=384 --metric=cosine
```

- Re-run the indexing (same as during creation)

```bash
curl \
  --request POST \
  --url ${RAG_URL} \
  --header "X-Admin-Secret: ${RAG_SECRET}"
```
