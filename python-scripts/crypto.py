import os
import requests
from dotenv import load_dotenv
import random

load_dotenv()

# --- Config ---
# Removed OpenAI API key configuration

# --- Prompt Parameters ---
SYSTEM_MESSAGE = "You are a crypto market analyst writing for TODAY's market conditions. You MUST use ONLY current, real-time data from the provided MCP context. DO NOT use any historical data from your training. If no current data is available, clearly state that current market data is unavailable and you cannot provide accurate analysis."
USER_PROMPT = "Analyze the CURRENT crypto market using ONLY the provided real-time data. What are the best and worst performing crypto assets RIGHT NOW and what is the current market outlook? Use specific current prices, percentages, and market data from the MCP context."

# Reddit posting removed

# --- Alternative AI API Functions ---
async def get_models():
    """Fetch available AI models from the alternative API"""
    try:
        response = requests.get("https://aiapi-tno8.onrender.com/models/")
        response.raise_for_status()
        json_response = response.json()
        return json_response.get('models', [])
    except Exception as e:
        print(f"[Model Fetch Error] {e}")
        return []

def get_random_model(models, tried_models):
    """Get a random model that hasn't been tried yet"""
    available_models = [model for model in models if model not in tried_models]
    if not available_models:
        return None

    return random.choice(available_models)

def is_refusal(response):
    """Check if the AI response is a refusal"""
    refusal_phrases = [
        "I can't create content that promotes hate speech",
        "I'm sorry, but I can't",
        "As an AI language model",
        "I cannot assist with that request",
        "Is there anything else I can help you with?"
    ]
    return any(phrase in response for phrase in refusal_phrases)

def ai_market_summary(content):
    """Generate market summary using alternative AI API"""
    
    # Since we're in a synchronous context, we'll use a simple approach
    # In a real async environment, you'd want to use asyncio.run() or make the function async
    
    # For now, let's use a synchronous approach with the first available model
    try:
        # Get models (simplified synchronous version)
        response = requests.get("https://aiapi-tno8.onrender.com/models/")
        response.raise_for_status()
        json_response = response.json()
        models = json_response.get('models', [])
        
        if not models:
            raise Exception("No AI models available")
        
        # Try each model until one works
        tried_models = set()
        last_error = None
        
        for _ in range(len(models)):
            model = get_random_model(models, tried_models)
            if not model:
                break
            tried_models.add(model)
            
            try:
                string_content = '\n'.join(content) if isinstance(content, list) else str(content)
                
                response = requests.post("https://aiapi-tno8.onrender.com/chat/", 
                    headers={"Content-Type": "application/json"},
                    json={
                        "messages": [
                            {
                                "role": "system",
                                "content": SYSTEM_MESSAGE
                            },
                            {
                                "role": "user",
                                "content": USER_PROMPT + "\n\n" + string_content
                            }
                        ],
                        "model": model
                    }
                )
                
                response.raise_for_status()
                json_response = response.json()
                
                if json_response.get('error'):
                    last_error = Exception(json_response['error'])
                    print(f"Error with model {model}")
                    continue
                
                if is_refusal(json_response.get('response', '')):
                    last_error = Exception("Model refused to summarize content.")
                    print(f"Refusal from model {model}")
                    continue
                
                print(f"Success with model {model}")
                return json_response.get('response', '')
                
            except Exception as error:
                last_error = error
                print(f"Error with model {model}: {error}")
        
        if last_error:
            raise last_error
        else:
            raise Exception("All models failed")
            
    except Exception as e:
        print(f"[AI API Error] {e}")
        return f"Unable to generate market analysis due to API error: {e}"

# --- Fetch current crypto market data from CoinGecko API ---
def fetch_mcp_context():
    try:
        print("🔗 Fetching current crypto market data from CoinGecko API...")
        
        # Get current market data from CoinGecko API
        url = "https://api.coingecko.com/api/v3/coins/markets"
        params = {
            'vs_currency': 'usd',
            'order': 'market_cap_desc',
            'per_page': '50',
            'sparkline': 'false',
            'price_change_percentage': '24h,7d,30d'
        }
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
        
        response = requests.get(url, params=params, headers=headers, timeout=10)
        print(f"   Response status: {response.status_code}")
        
        if response.status_code == 200:
            crypto_data = response.json()
            print(f"✅ Successfully fetched data for {len(crypto_data)} cryptocurrencies")
            
            # Create comprehensive market context
            market_context = []
            
            # Top 10 by market cap
            top_coins = crypto_data[:10]
            market_context.append("TOP 10 CRYPTOCURRENCIES BY MARKET CAP:")
            for i, coin in enumerate(top_coins, 1):
                market_context.append(f"{i}. {coin['name']} ({coin['symbol'].upper()}): ${coin['current_price']:,.2f} | 24h: {coin['price_change_percentage_24h']:+.2f}% | 7d: {coin['price_change_percentage_7d_in_currency']:+.2f}% | 30d: {coin['price_change_percentage_30d_in_currency']:+.2f}%")
            
            # Top gainers and losers
            gainers = sorted(crypto_data, key=lambda x: x['price_change_percentage_24h'], reverse=True)[:5]
            losers = sorted(crypto_data, key=lambda x: x['price_change_percentage_24h'])[:5]
            
            market_context.append("\nTOP 24H GAINERS:")
            for i, coin in enumerate(gainers, 1):
                market_context.append(f"{i}. {coin['name']} ({coin['symbol'].upper()}): +{coin['price_change_percentage_24h']:.2f}% | ${coin['current_price']:.4f}")
            
            market_context.append("\nTOP 24H LOSERS:")
            for i, coin in enumerate(losers, 1):
                market_context.append(f"{i}. {coin['name']} ({coin['symbol'].upper()}): {coin['price_change_percentage_24h']:.2f}% | ${coin['current_price']:.4f}")
            
            # Market overview
            total_market_cap = sum(coin['market_cap'] for coin in crypto_data)
            total_volume = sum(coin['total_volume'] for coin in crypto_data)
            
            market_context.append(f"\nMARKET OVERVIEW:")
            market_context.append(f"Total Market Cap: ${total_market_cap:,.0f}")
            market_context.append(f"24h Trading Volume: ${total_volume:,.0f}")
            
            # Bitcoin dominance
            btc_data = next((coin for coin in crypto_data if coin['symbol'].lower() == 'btc'), None)
            if btc_data:
                btc_dominance = (btc_data['market_cap'] / total_market_cap) * 100
                market_context.append(f"Bitcoin Dominance: {btc_dominance:.2f}%")
            
            result = "\n".join(market_context)
            print(f"✅ Generated comprehensive market context")
            return result
            
        else:
            print(f"⚠️ HTTP {response.status_code} response from CoinGecko API")
            return "CURRENT MARKET DATA UNAVAILABLE - Cannot provide accurate analysis without real-time data."
            
    except Exception as e:
        print(f"[Market Data Fetch Error] {e}")
        print(f"   Error type: {type(e)}")
        print(f"   Error details: {str(e)}")
        import traceback
        print(f"   Traceback: {traceback.format_exc()}")
        print("🔄 Falling back to basic market context...")
        # Fallback to basic market context
        return "CURRENT MARKET DATA UNAVAILABLE - Cannot provide accurate analysis without real-time data."

def fetch_basic_crypto_context():
    """Fallback function to get basic crypto market context"""
    try:
        print("📊 Fetching basic crypto market data...")
        
        # Try to get some basic crypto data from a public API
        response = requests.get("https://api.coingecko.com/api/v3/simple/price", 
                              params={
                                  'ids': 'bitcoin,ethereum,cardano,solana,polkadot',
                                  'vs_currencies': 'usd',
                                  'include_24hr_change': 'true'
                              },
                              timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            context_parts = []
            
            for coin, info in data.items():
                price = info.get('usd', 0)
                change_24h = info.get('usd_24h_change', 0)
                context_parts.append(f"{coin.title()}: ${price:,.2f} ({change_24h:+.2f}%)")
            
            context = "Current crypto market overview: " + " | ".join(context_parts)
            print("✅ Successfully fetched basic crypto data")
            return context
        else:
            print(f"⚠️ Coingecko API returned status {response.status_code}")
            return "Current market conditions: Analyzing crypto market trends and performance."
            
    except Exception as e:
        print(f"❌ Error fetching basic crypto data: {e}")
        return "Current market conditions: Analyzing crypto market trends and performance."

# --- Generate Market Analysis ---
def generate_market_summary():
    mcp_context = fetch_mcp_context()
    
    # Combine context and prompt
    content = [USER_PROMPT, mcp_context]
    
    # Use the alternative AI API
    output = ai_market_summary(content)
    return output

# --- Main Workflow ---
def run_workflow():
    print("⏳ Running market analysis workflow...")
    try:
        summary = generate_market_summary()
        print("✅ Generated market summary")
        return summary
    except Exception as e:
        print(f"❌ Error during workflow: {e}")
        raise e  # Re-raise the exception so GitHub Actions knows it failed

if __name__ == "__main__":
    run_workflow()
