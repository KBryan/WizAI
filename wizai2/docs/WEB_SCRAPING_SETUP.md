# Web Scraping Setup Guide

This guide explains how to set up web scraping capabilities for your Real Estate Researcher agent.

## Overview

The web scraping module enables your Real Estate Researcher to:
- Search Google News for current market articles
- Monitor Reddit discussions for community sentiment
- Get real-time data on inventory, prices, and trends
- Track policy changes and regulatory news

## Supported Providers

### 1. SerpAPI (Recommended)

**Best for:** Google Search, Google News

**Pricing:**
- Free: 100 searches/month
- Hobby: $50/month (5,000 searches)
- Business: $130/month (20,000 searches)

**Setup:**
1. Sign up at [serpapi.com](https://serpapi.com)
2. Get your API key from the dashboard
3. Add to `.env` file:
```bash
SERPAPI_KEY=your_api_key_here
```

**Features:**
- Google Search results
- Google News
- Structured data extraction
- Rate limiting
- Cached results

### 2. ScrapingBee (Alternative)

**Best for:** Direct web scraping with JavaScript rendering

**Pricing:**
- Free trial: 1,000 API credits
- Freelancer: $49/month (100,000 credits)
- Business: $99/month (500,000 credits)

**Setup:**
1. Sign up at [scrapingbee.com](https://www.scrapingbee.com)
2. Get your API key
3. Add to `.env` file:
```bash
SCRAPINGBEE_KEY=your_api_key_here
```

**Features:**
- Direct web scraping
- JavaScript rendering
- Proxy rotation
- CAPTCHA handling

## Configuration

### Step 1: Choose Provider

Edit `spree/.env`:

```bash
# Option 1: SerpAPI (Recommended for news search)
SERPAPI_KEY=your_serpapi_key_here

# Option 2: ScrapingBee
SCRAPINGBEE_KEY=your_scrapingbee_key_here

# Optional: Rate limiting (requests per minute)
WEB_SCRAPING_RATE_LIMIT=60

# Optional: Cache duration (seconds)
WEB_SCRAPING_CACHE_DURATION=3600
```

### Step 2: Restart Server

```bash
cd wizai2
cargo run
```

You should see in the logs:
```
Web scraping configured: SerpAPI
```

## Usage Examples

### Example 1: Search Market News

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "YOUR_RESEARCHER_ID",
    "task": "Search for recent news about Durham Region real estate market"
  }'
```

### Example 2: Get Market Sentiment

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "YOUR_RESEARCHER_ID",
    "task": "What is the current market sentiment for Whitby?"
  }'
```

### Example 3: Check Reddit Discussions

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "YOUR_RESEARCHER_ID",
    "task": "Search Reddit for discussions about Oshawa real estate"
  }'
```

## Real Estate Research Queries

The researcher agent understands these types of queries:

### Market Analysis
- "Search for news about [location] real estate market"
- "Get current market sentiment for [location]"
- "What are people saying about [location] housing on Reddit?"

### Specific Data
- "Find articles about [location] inventory levels"
- "Search for [location] days on market trends"
- "Get news about interest rates affecting [location]"

### Policy/Regulatory
- "Find news about foreign buyer ban in [location]"
- "Search for policy changes affecting [location] real estate"
- "Get updates on mortgage rules"

## Data Sources

When configured, the agent searches:

### News Sources
- **DurhamRegion.com** - Local news
- **Toronto Star** - Regional coverage
- **Globe and Mail** - National analysis
- **Better Dwelling** - Market analysis
- **Google News** - Aggregated articles

### Community Sources
- **Reddit r/realestatecanada** - National discussions
- **Reddit r/PersonalFinanceCanada** - Financial advice
- **Reddit r/TorontoRealEstate** - Local market talk

### Government Sources
- **Bank of Canada** - Interest rate announcements
- **CMHC** - Housing market reports
- **Local real estate boards** - Official statistics

## Rate Limits

Default: 60 requests per minute

**To change:**
```bash
# In .env
WEB_SCRAPING_RATE_LIMIT=100
```

## Caching

Results are cached for 1 hour by default to:
- Reduce API costs
- Improve response time
- Avoid duplicate searches

**To change:**
```bash
# In .env
WEB_SCRAPING_CACHE_DURATION=1800  # 30 minutes
```

## Cost Estimates

### SerpAPI
- **News search:** 1 credit per search
- **100 searches/month:** Free
- **1,000 searches/month:** ~$10 (Hobby plan)
- **10,000 searches/month:** ~$50 (Hobby plan)

### ScrapingBee
- **Simple scrape:** 1 credit
- **With JavaScript:** 5 credits
- **100,000 credits:** $49/month

**Typical usage:**
- 1 research report = 5-10 searches
- 100 reports/month = ~500 searches
- Cost: $10-20/month

## Troubleshooting

### Issue: "Web scraping not configured"

**Solution:**
1. Check `.env` file exists in `spree/` directory
2. Verify API key is set: `SERPAPI_KEY=your_key`
3. Restart the server

### Issue: "Rate limit exceeded"

**Solution:**
1. Wait 1 minute and try again
2. Increase rate limit in `.env`
3. Upgrade your API plan

### Issue: "No results found"

**Solution:**
1. Check your search query is specific enough
2. Try different keywords
3. Verify the location name (e.g., "Whitby, Ontario" vs "Whitby")

### Issue: "API error"

**Solution:**
1. Check API key is valid
2. Verify you have credits remaining
3. Check service status (SerpAPI/ScrapingBee status pages)

## Testing

Test your configuration:

```bash
# Test web scraping directly
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "YOUR_RESEARCHER_ID",
    "task": "Search web for Durham Region real estate news"
  }'
```

## Next Steps

1. **Get API Key:** Sign up for SerpAPI (recommended) or ScrapingBee
2. **Configure:** Add key to `spree/.env`
3. **Restart:** Run `cargo run` in spree directory
4. **Test:** Send a research query to your agent
5. **Monitor:** Check logs for web scraping activity

## Support

- **SerpAPI Docs:** https://serpapi.com/search-api
- **ScrapingBee Docs:** https://www.scrapingbee.com/documentation/
- **GitHub Issues:** https://github.com/your-repo/issues

---

**Questions?** The Real Estate Researcher is now ready to search the web for current market data!
