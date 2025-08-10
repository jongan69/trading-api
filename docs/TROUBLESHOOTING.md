# Trading API Troubleshooting Guide

This guide helps resolve common issues encountered when running the trading API.

## Common Issues and Solutions

### 1. HTTP 502 Bad Gateway Errors

**Symptoms:**
```
ERROR tower_http::trace::on_failure: response failed classification=Status code: 502 Bad Gateway
```

**Causes:**
- External API services (Yahoo Finance, Alpaca, Reddit) are down or unreachable
- Network connectivity issues
- Rate limiting from external services

**Solutions:**
1. **Check external service status:**
   ```bash
   curl http://localhost:3000/status
   ```

2. **Verify network connectivity:**
   ```bash
   curl -I https://api.alpaca.markets/v2/clock
   curl -I https://query1.finance.yahoo.com/v8/finance/chart/AAPL
   ```

3. **Enable retry logic** (already implemented):
   - Set `RETRY_ENABLED=true` in your environment
   - Adjust retry settings:
     ```bash
     export RETRY_MAX_RETRIES=5
     export RETRY_BASE_DELAY_MS=2000
     export RETRY_MAX_DELAY_MS=30000
     ```

### 2. HTTP 429 Too Many Requests (Rate Limiting)

**Symptoms:**
```
[options] yahoo predefined error: yahoo predefined status 429 Too Many Requests
```

**Causes:**
- Exceeding API rate limits for Yahoo Finance, Alpaca, or other services
- Too many concurrent requests

**Solutions:**
1. **Enable rate limiting:**
   ```bash
   export RATE_LIMIT_ENABLED=true
   export RATE_LIMIT_REQUESTS_PER_MINUTE=30
   export RATE_LIMIT_BURST_SIZE=5
   ```

2. **Reduce concurrency:**
   ```bash
   # In your .env file or environment
   export CONCURRENCY_LIMIT=4  # Reduce from default 8
   ```

3. **Add delays between requests:**
   - The API now includes automatic backoff and retry logic
   - Consider implementing request queuing for high-traffic scenarios

### 3. Invalid Ticker Symbol Errors

**Symptoms:**
```
Error fetching short-term contracts for FSKAX: Invalid ticker symbol: FSKAX
Error fetching short-term contracts for FZILX: Invalid ticker symbol: FZILX
```

**Causes:**
- Mutual fund symbols (FSKAX, FZILX) are not supported by options APIs
- Invalid or malformed ticker symbols
- Symbols that don't exist on the target exchange

**Solutions:**
1. **Filter out unsupported symbols:**
   - The API now includes symbol validation
   - Mutual funds and ETFs may not have options available
   - Use the `/status` endpoint to check service health

2. **Validate symbols before processing:**
   ```rust
   use crate::utils::validate_ticker_symbol;
   
   for symbol in symbols {
       if let Err(e) = validate_ticker_symbol(&symbol) {
           tracing::warn!("Invalid symbol {}: {}", symbol, e);
           continue;
       }
       // Process valid symbol
   }
   ```

### 4. HTML Parsing Errors

**Symptoms:**
```
Fail to parse tag a
```

**Causes:**
- Changes in website structure (Reddit, Finviz, etc.)
- Anti-bot measures
- Network issues causing incomplete HTML responses

**Solutions:**
1. **Update scraping logic:**
   - Check if target websites have changed their structure
   - Implement more robust HTML parsing with fallbacks

2. **Add error handling:**
   ```rust
   match parse_html_content(&html) {
       Ok(content) => process_content(content),
       Err(e) => {
           tracing::warn!("Failed to parse HTML: {}", e);
           // Use fallback data source or skip
       }
   }
   ```

3. **Implement retry with different user agents:**
   - Rotate user agent strings
   - Add delays between requests

### 5. Service Health Monitoring

**New Health Check Endpoints:**

1. **Basic Health Check:**
   ```bash
   curl http://localhost:3000/health
   ```

2. **Detailed System Status:**
   ```bash
   curl http://localhost:3000/status
   ```

3. **Readiness Check (for Kubernetes):**
   ```bash
   curl http://localhost:3000/ready
   ```

4. **Liveness Check:**
   ```bash
   curl http://localhost:3000/live
   ```

5. **System Metrics:**
   ```bash
   curl http://localhost:3000/metrics
   ```

### 6. Environment Configuration

**Recommended Environment Variables:**

```bash
# Server Configuration
PORT=3000
HOST=0.0.0.0
RUST_LOG=info

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST_SIZE=10

# Retry Configuration
RETRY_ENABLED=true
RETRY_MAX_RETRIES=3
RETRY_BASE_DELAY_MS=1000
RETRY_MAX_DELAY_MS=10000

# Concurrency
CONCURRENCY_LIMIT=8

# External API Keys
ALPACA_API_KEY_ID=your_key_here
ALPACA_API_SECRET_KEY=your_secret_here

# Optional Reddit Configuration
REDDIT_CLIENT_ID=your_reddit_client_id
REDDIT_CLIENT_SECRET=your_reddit_client_secret
REDDIT_USERNAME=your_reddit_username
REDDIT_PASSWORD=your_reddit_password
```

### 7. Debugging Tips

1. **Enable Debug Logging:**
   ```bash
   export RUST_LOG=debug
   ```

2. **Check API Documentation:**
   ```bash
   # Open in browser
   open http://localhost:3000/docs
   ```

3. **Monitor Real-time Logs:**
   ```bash
   cargo run 2>&1 | tee api.log
   ```

4. **Test Individual Endpoints:**
   ```bash
   # Test options recommendations
   curl "http://localhost:3000/options/recommendations?limit=5&debug=true"
   
   # Test trending stocks
   curl "http://localhost:3000/data/trending/stocks?limit=10"
   ```

### 8. Performance Optimization

1. **Reduce Concurrency for Stability:**
   ```bash
   export CONCURRENCY_LIMIT=4
   ```

2. **Increase Timeouts:**
   ```bash
   export REQUEST_TIMEOUT_MS=30000
   ```

3. **Enable Connection Pooling:**
   - The HTTP client now includes connection pooling
   - Adjust pool size based on your needs

### 9. Monitoring and Alerting

1. **Set up Health Checks:**
   ```bash
   # Cron job to check health every minute
   */1 * * * * curl -f http://localhost:3000/health || echo "API is down"
   ```

2. **Monitor Error Rates:**
   ```bash
   # Check error rate in logs
   tail -f api.log | grep "ERROR"
   ```

3. **Track Response Times:**
   ```bash
   # Monitor slow requests
   tail -f api.log | grep "Slow request"
   ```

### 10. Getting Help

If you continue to experience issues:

1. **Check the logs** for specific error messages
2. **Verify your API keys** are valid and have proper permissions
3. **Test external services** directly to ensure they're accessible
4. **Review the system status** using the new `/status` endpoint
5. **Check the API documentation** at `/docs` for endpoint details

For persistent issues, please provide:
- Complete error logs
- System status output
- Environment configuration (without sensitive keys)
- Steps to reproduce the issue
