# Chatbot Backend

The NFTREASURE Community ChatBot

This repository contains the source files for the Ron Jay [ChatBot](https://chatbot.nftreasure.community) backend.

This is an unofficial chatbot for the NFTREASURE Community and is not affiliated with the NFTREASURE project.

It is made with :heart: _love_ :heart: for the NFTREASURE Community.

## How to run locally

```bash
# Get the dependencies
just -q init

# Login to Cloudflare
just -q login

# Build the local workers
just -q build

# Start the local workers
just -q start

# Stop the local workers
just -q stop
```

## How to test locally

There are a few URLs that you can use to test the chatbot locally depending on the worker.

### Caddy

Caddy Server can be used to proxy the chatbot requests to the correct worker.

One you have started the script from the frontend repository, access Caddy at: [https://localhost:9000](https://localhost:9000)

### Frontend

The frontend worker is managed in a separate repository and is responsible for rendering the chatbot UI using Astro.

The source code for the frontend worker can be found [here](https://github.com/NFTREASURE-Community/chatbot-frontend).

Access to the Frontend Worker is via Caddy at: [https://localhost:9000](https://localhost:9000) -> [http://localhost:9100](http://localhost:9100)

The frontend worker embeds the UI Widget worker on he main page behind a button or offers a fullscreen page at /fullscreen.

### Backend

The backend worker is managed in this repository.

Once you have the chatbot backend running, access to the Worker is via Caddy at: [https://localhost:9000/ai](https://localhost:9000/ai) -> [http://localhost:9200](http://localhost:9200)
