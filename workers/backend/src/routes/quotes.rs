use utils::console_trace;
use utils::handler::extract_query_param;

use worker::*;

use rand::seq::SliceRandom;
use serde::Serialize;

#[derive(Serialize)]
struct QuoteResponse {
    quote: String,
}

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

static QUOTES_CRYPTO: &[&str] = &[
  "Lost in the crypto maze? Let's find that lambo together, degen!",
  "In crypto we trust, even when we're dust.",
  "Buy high, sell low? You're in the right place, bro!",
  "Fiat is so last century. Welcome to the future.",
  "Not all who wander are lost, but I definitely am in this market.",
  "Keep calm and HODL on.",
  "Roses are red, violets are blue, if you don't believe in crypto, I don't believe in you.",
  "Satoshi Nakamoto for President.",
  "Why go to the moon when we can go to Mars?",
  "A day without crypto is like... just kidding, I have no idea.",
  "Bitcoin: because we all need an expensive digital rock.",
  "If you can't handle my 40% drops, you don't deserve my 400% gains.",
  "I'm in it for the technology. And the lambo.",
  "Don't trust, verify. Then HODL.",
  "You had me at 'decentralized'.",
  "Life's a blockchain, make every transaction count.",
  "Altcoins: because why have one risky investment when you can have many?",
  "Mining Bitcoin with my heart since 2013.",
  "Stablecoins? More like bored-money.",
  "I told my mom I was buying digital gold. Now, she's mining Ethereum.",
  "Crypto: breaking hearts and wallets since 2009.",
  "To the moon or to the noodle shop?",
  "Bear markets are just shopping opportunities in disguise.",
  "I didn't choose the crypto life, the crypto life chose me.",
  "May your coffee be strong and your block confirmations be swift.",
  "My portfolio's in the red, but my spirits are in the green.",
  "Here for a good time, not a long time—unless we're talking about block times.",
  "Proof of Stake? More like proof of steak—I'm here for the beefy gains.",
  "They said crypto's a rollercoaster, but I never liked amusement parks anyway.",
  "Diversification in crypto just means choosing different colors for my lambo.",
  "Some people meditate; I check crypto prices.",
  "Asking me for financial advice? That's like asking a fish how to fly.",
  "My crypto's under water, but I'm still breathing.",
  "Decentralize everything, except pizza. That, you can send straight to me.",
  "I'm not addicted to crypto; I can quit as soon as I recover my losses.",
  "Wallets are like onions; they both have layers and can make you cry.",
  "Is it really a loss if I refuse to sell?",
  "I came for the gains but stayed for the memes.",
  "Crypto's giving me trust issues.",
  "Can't spell 'cryptocurrency' without 'cry'.",
  "Remember, kids, only invest what you're willing to forget.",
  "This isn't volatility, it's just 'price excitement'.",
  "I have a diversified portfolio—Bitcoin and Bitcoin memes.",
  "I miss the days when 'hash' was just something I had for breakfast.",
  "My retirement plan is a cold wallet in a hot market.",
  "You say 'market correction', I say 'discount shopping'.",
  "My other car was a NFT.",
  "If you don't believe it or don't get it, I don't have the time to try to convince you, sorry. - Satoshi Nakamoto",
  "Every time a coin gets listed, an angel gets its wings.",
  "Crypto: where the predictions are made up and the market cap doesn't matter.",
  "Who doesn't rug you makes you stronger.",
  "A rug pull a day keeps your gains away.",
];

pub async fn handle_get(req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    console_trace!("TRACE: Request received for route 'quotes'");

    let url = req.url()?;

    // Extract the query parameters or provide the default value.
    let quote_type = extract_query_param(&url, "type", "quote").await;
    console_trace!("TRACE: Extracted query parameters: type: {}", quote_type);

    // Based on the quote type, choose a random quote.
    let quote: String = match quote_type.as_str() {
        "quote" => QUOTES
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"Keep calm and carry on.")
            .to_string(),
        "crypto" => QUOTES_CRYPTO
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string(),
        _ => format!("Unknown quote type {}. Please try again later.", quote_type),
    };

    let quote_response = QuoteResponse { quote };

    // Create the JSON response
    let mut response = Response::from_json(&quote_response)?;

    // Add required headers to the response.
    let headers = response.headers_mut();
    headers.set("Content-Type", "application/json; charset=utf-8")?;

    // Add CORS headers to the response and return
    //add_headers(&mut response, origin)?;
    Ok(response)
}
