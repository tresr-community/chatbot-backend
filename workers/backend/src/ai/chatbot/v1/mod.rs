use chatbot_utils::console_trace;

use worker::*;

use rand::seq::SliceRandom;
use serde_json::json;

static QUOTES: &[&str] = &[
    "Ron Jay is currently FIAT mining right now. Please try again later.",
    "Ron Jay is currently on a moon mission. Please try again later.",
    "Ron Jay is currently on a noodle break. Please try again later.",
    "Ron Jay is currently on a coffee break. Please try again later.",
    "Ron Jay is currently on a lunch break. Please try again later.",
    "Ron Jay is currently farming air drops. Please try again later.",
    "Ron Jay is currently staking his claim. Please try again later.",
    "Ron Jay is currently HODLing. Please try again later.",
    "Ron Jay is currently on a shopping spree. Please try again later.",
    "Ron Jay is currently on a meme break. Please try again later.",
    "Ron Jay is currently protesting his poor pay conditions. Please try again later.",
    "Ron Jay is currently apeing into a new project. Please try again later.",
    "Ron Jay is currently chasing blockchains. Please try again later.",
    "Ron Jay is currently decoding smart contracts. Please try again later.",
    "Ron Jay is currently diving deep into the DeFi ocean. Please try again later.",
    "Ron Jay is currently exploring the NFT jungle. Please try again later.",
    "Ron Jay is currently navigating through crypto volatility. Please try again later.",
    "Ron Jay is currently calculating gas fees. Please try again later.",
    "Ron Jay is currently attending a blockchain conference. Please try again later.",
    "Ron Jay is currently debating crypto vs. fiat. Please try again later.",
    "Ron Jay is currently seeking Satoshi's wisdom. Please try again later.",
    "Ron Jay is currently in a DAO meeting. Please try again later.",
    "Ron Jay is currently optimizing his mining rig. Please try again later.",
    "Ron Jay is currently drafting whitepapers. Please try again later.",
    "Ron Jay is currently arbitraging across exchanges. Please try again later.",
    "Ron Jay is currently hodling through the dip. Please try again later.",
    "Ron Jay is currently predicting the next bull run. Please try again later.",
    "Ron Jay is currently exploring Layer 2 solutions. Please try again later.",
    "Ron Jay is currently unboxing a rare NFT. Please try again later.",
    "Ron Jay is currently researching the latest ICO. Please try again later.",
    "Ron Jay is currently mastering yield farming. Please try again later.",
    "Ron Jay is currently safeguarding his private keys. Please try again later.",
    "Ron Jay is currently tracing transaction hashes. Please try again later.",
    "Ron Jay is currently on a quest for the next Bitcoin. Please try again later.",
    "Ron Jay is currently auditing smart contracts for fun. Please try again later.",
    "Ron Jay is currently scaling crypto mountains. Please try again later.",
    "Ron Jay is currently swapping tokens on a DEX. Please try again later.",
];

pub async fn handle_ai(message: &str) -> worker::Result<Response> {
    console_trace!("TRACE: Handling request to AI backend 'chatbot'");

    // Select a random quote from the list.
    let quote: String = QUOTES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"Keep calm and carry on.")
        .to_string();

    // Construct a JSON payload to return.
    let payload = json!({
        "message": message,
        "reply": quote,
        "error": "",
    });
    let body = serde_json::to_string(&payload).unwrap();

    // Creating a new headers map
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json; charset=utf-8")?;

    Response::ok(body).map(|resp| resp.with_headers(headers))
}
