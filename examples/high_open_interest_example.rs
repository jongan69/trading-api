use trading_api::helpers::high_open_interest::get_high_open_interest_contracts;
use trading_api::helpers::high_open_interest::get_high_open_interest_contracts_batch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    println!("=== High Open Interest Contracts Example ===\n");

    // Example 1: Get high open interest contracts for a single ticker
    println!("1. Getting high open interest contracts for AAPL (calls):");
    let aapl_result = get_high_open_interest_contracts("AAPL", Some("call")).await;
    
    match &aapl_result.short_term {
        Some(contract) => {
            println!("   Short-term contract:");
            println!("     Symbol: {}", contract.symbol);
            println!("     Strike: ${}", contract.strike_price);
            println!("     Expiration: {}", contract.expiration_date);
            println!("     Open Interest: {:?}", contract.open_interest);
            println!("     Ask Price: ${:?}", contract.ask_price);
            println!("     Bid Price: ${:?}", contract.bid_price);
        }
        None => println!("   No short-term contracts found"),
    }

    match &aapl_result.leap {
        Some(contract) => {
            println!("   LEAP contract:");
            println!("     Symbol: {}", contract.symbol);
            println!("     Strike: ${}", contract.strike_price);
            println!("     Expiration: {}", contract.expiration_date);
            println!("     Open Interest: {:?}", contract.open_interest);
            println!("     Ask Price: ${:?}", contract.ask_price);
            println!("     Bid Price: ${:?}", contract.bid_price);
        }
        None => println!("   No LEAP contracts found"),
    }

    println!("\n2. Getting high open interest contracts for TSLA (puts):");
    let tsla_result = get_high_open_interest_contracts("TSLA", Some("put")).await;
    
    match &tsla_result.short_term {
        Some(contract) => {
            println!("   Short-term contract:");
            println!("     Symbol: {}", contract.symbol);
            println!("     Strike: ${}", contract.strike_price);
            println!("     Expiration: {}", contract.expiration_date);
            println!("     Open Interest: {:?}", contract.open_interest);
            println!("     Ask Price: ${:?}", contract.ask_price);
            println!("     Bid Price: ${:?}", contract.bid_price);
        }
        None => println!("   No short-term contracts found"),
    }

    match &tsla_result.leap {
        Some(contract) => {
            println!("   LEAP contract:");
            println!("     Symbol: {}", contract.symbol);
            println!("     Strike: ${}", contract.strike_price);
            println!("     Expiration: {}", contract.expiration_date);
            println!("     Open Interest: {:?}", contract.open_interest);
            println!("     Ask Price: ${:?}", contract.ask_price);
            println!("     Bid Price: ${:?}", contract.bid_price);
        }
        None => println!("   No LEAP contracts found"),
    }

    println!("\n3. Getting high open interest contracts for multiple tickers (batch):");
    let tickers = vec!["AAPL".to_string(), "TSLA".to_string(), "META".to_string()];
    let batch_results = get_high_open_interest_contracts_batch(&tickers, Some("call")).await;
    
    for (ticker, result) in batch_results {
        println!("   {ticker}:");
        if let Some(contract) = &result.short_term {
            println!("     Short-term: {} @ ${} (OI: {:?})", 
                contract.symbol, contract.strike_price, contract.open_interest);
        }
        if let Some(contract) = &result.leap {
            println!("     LEAP: {} @ ${} (OI: {:?})", 
                contract.symbol, contract.strike_price, contract.open_interest);
        }
    }

    println!("\n=== Example completed ===");
    Ok(())
}
