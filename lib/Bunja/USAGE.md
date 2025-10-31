# Bunja Usage Guide

## Getting Started

### Installation

1. Clone the repository and build:

```bash
cargo build --release
```

2. The binary will be available at `./target/release/bunja`

Optionally, install it globally:

```bash
cargo install --path .
```

## Commands

### init - Initialize Configuration

Create a new `bunja.lock` configuration file with default settings.

```bash
bunja init
```

Optional: Specify a custom path:

```bash
bunja init --path /path/to/bunja.lock
```

**What it does:**
- Creates a `bunja.lock` file with Pexels and Unsplash domains pre-configured
- Sets up default cache and server settings
- Ready to use after adding API keys

### add - Add Asset Domain

Add a new asset domain to your configuration.

```bash
bunja add <name> <provider> <base_url> [--api-key KEY]
```

**Examples:**

```bash
# Pexels
bunja add my-pexels pexels https://api.pexels.com/v1 --api-key YOUR_PEXELS_KEY

# Unsplash
bunja add my-unsplash unsplash https://api.unsplash.com --api-key YOUR_UNSPLASH_KEY

# Cloudinary
bunja add my-cloudinary cloudinary https://res.cloudinary.com/YOUR_CLOUD

# S3
bunja add my-s3 s3 https://bucket.s3.region.amazonaws.com

# Custom URL
bunja add my-cdn custom https://cdn.example.com/images

# Local filesystem
bunja add local-imgs local ./public/images
```

**Providers:**
- `pexels` - Pexels API
- `unsplash` - Unsplash API
- `cloudinary` - Cloudinary CDN
- `s3` - S3 or compatible storage
- `custom` - Any HTTP(S) endpoint
- `local` - Local filesystem

### remove - Remove Asset Domain

Remove a domain from configuration.

```bash
bunja remove <name>
```

**Example:**

```bash
bunja remove my-pexels
```

### list - List Domains

Display all configured domains.

```bash
bunja list
```

**Output:**

```
Configured domains:
  pexels (Pexels)
    Base URL: https://api.pexels.com/v1
    API Key: [configured]
  unsplash (Unsplash)
    Base URL: https://api.unsplash.com
    API Key: [configured]
```

### serve - Start HTTP Server

Start the Bunja HTTP server to serve assets.

```bash
bunja serve
```

Optional: Override port from config:

```bash
bunja serve --port 3000
```

**Server Endpoints:**

- `GET /bunja/{domain}/{path}` - Fetch and serve asset
- `GET /health` - Health check
- `GET /api/cache/stats` - Cache statistics
- `POST /api/cache/clear` - Clear cache

**Example requests:**

```bash
# Fetch from Pexels
curl http://localhost:8080/bunja/pexels/sunset

# Fetch from S3
curl http://localhost:8080/bunja/my-s3/images/logo.png

# Cache stats
curl http://localhost:8080/api/cache/stats

# Clear cache
curl -X POST http://localhost:8080/api/cache/clear
```

### translate - Translate Files

Translate `bunja://` asset calls in a file to actual URLs.

```bash
bunja translate --input <file> --output <file>
```

**Example:**

```bash
bunja translate --input index.html --output index.translated.html
```

**Before (index.html):**

```html
<img src="bunja://pexels/nature" alt="Nature">
<div style="background: url('bunja://unsplash/mountain')"></div>
```

**After (index.translated.html):**

```html
<img src="/bunja/pexels/nature" alt="Nature">
<div style="background: url('/bunja/unsplash/mountain')"></div>
```

### prefetch - Prefetch Assets

Scan a directory and prefetch all `bunja://` assets to cache.

```bash
bunja prefetch <directory>
```

**Example:**

```bash
bunja prefetch ./public
```

**What it does:**
- Recursively scans directory
- Finds all `.html`, `.css`, and `.js` files
- Extracts `bunja://` asset calls
- Fetches and caches all assets

**Use cases:**
- Pre-populate cache before deployment
- Warm up cache after clearing
- Build-time asset optimization

### cache - Cache Management

#### Show Statistics

Display cache statistics.

```bash
bunja cache stats
```

**Output:**

```
Cache Statistics:
  Total entries: 156
  Total size: 342 MB
  Max size: 1024 MB
  Usage: 33.40%
```

#### Clear Cache

Clear all cached assets.

```bash
bunja cache clear
```

### validate - Validate Configuration

Validate your `bunja.lock` configuration file.

```bash
bunja validate
```

**Checks:**
- File syntax is valid TOML
- All required fields are present
- Domains have valid configurations
- Base URLs are not empty

**Output:**

```
bunja.lock is valid!
  Version: 1.0.0
  Domains: 6
  Cache directory: .bunja_cache
  Server port: 8080
```

## Asset Call Syntax

Bunja recognizes asset calls in multiple formats:

### Protocol Format

```
bunja://<domain>/<path>
```

**Examples:**

```html
<!-- HTML -->
<img src="bunja://pexels/sunset">
<video src="bunja://s3/videos/intro.mp4">

<!-- CSS -->
.hero {
    background-image: url('bunja://unsplash/mountains');
}

<!-- JavaScript -->
const img = 'bunja://cloudinary/hero.jpg';
import photo from 'bunja://local/photo.png';
```

### Domain Names

The domain name must match a configured domain in `bunja.lock`.

### Asset Paths

The path part depends on the provider:

**Search-based (Pexels, Unsplash):**
- Path is used as search query
- Returns first matching image
- Example: `bunja://pexels/nature` searches for "nature"

**URL-based (S3, Cloudinary, Custom, Local):**
- Path is appended to base URL
- Example: `bunja://s3/images/logo.png` → `https://bucket.s3...com/images/logo.png`

## Configuration Deep Dive

### Global Settings

```toml
[global_settings]
cache_dir = ".bunja_cache"          # Cache directory path
max_cache_size_mb = 1024            # Maximum cache size in MB
cache_ttl_seconds = 86400           # Cache TTL (24 hours)
enable_compression = false          # Enable response compression
worker_threads = 4                  # HTTP server worker threads
server_port = 8080                  # HTTP server port
enable_https = false                # HTTPS (not yet implemented)
log_level = "info"                  # Log level
```

### Domain Configuration

```toml
[domains.my-domain]
provider = "pexels"                         # Provider type
base_url = "https://api.pexels.com/v1"     # Base URL
api_key = "YOUR_KEY"                        # Optional API key
fallback_domains = ["other-domain"]         # Fallback domains

# Optional: Custom headers
[domains.my-domain.headers]
"X-Custom-Header" = "value"

# Optional: Rate limiting
[domains.my-domain.rate_limit]
requests_per_second = 20
burst_size = 50

# Optional: Retry strategy
[domains.my-domain.retry_strategy]
max_retries = 3
backoff_ms = 1000
exponential_backoff = true

# Optional: Transformations (Cloudinary)
[[domains.my-domain.transformations]]
name = "thumbnail"
[domains.my-domain.transformations.parameters]
w = "300"
h = "300"
c = "fill"
```

## Workflow Examples

### Local Development

1. Initialize:

```bash
bunja init
```

2. Add local assets:

```bash
bunja add local local ./assets
```

3. Start server:

```bash
bunja serve
```

4. Use in HTML:

```html
<img src="bunja://local/logo.png">
```

5. Access via server:

```
http://localhost:8080/bunja/local/logo.png
```

### Production Deployment

1. Configure production domains:

```bash
bunja add prod-s3 s3 https://prod-bucket.s3.amazonaws.com
bunja add prod-cdn custom https://cdn.example.com
```

2. Translate files:

```bash
bunja translate --input src/index.html --output dist/index.html
```

3. Prefetch assets:

```bash
bunja prefetch ./dist
```

4. Start server:

```bash
bunja serve --port 8080
```

### Multi-Provider Fallback

Configure fallback domains for reliability:

```toml
[domains.primary]
provider = "pexels"
base_url = "https://api.pexels.com/v1"
api_key = "PRIMARY_KEY"
fallback_domains = ["secondary", "tertiary"]

[domains.secondary]
provider = "unsplash"
base_url = "https://api.unsplash.com"
api_key = "SECONDARY_KEY"
fallback_domains = ["tertiary"]

[domains.tertiary]
provider = "custom"
base_url = "https://backup-cdn.example.com"
```

If `primary` fails, Bunja tries `secondary`, then `tertiary`.

## Performance Tips

1. **Prefetch assets** before production deployment
2. **Increase cache size** for high-traffic sites
3. **Use local provider** for static assets
4. **Configure rate limits** to avoid API throttling
5. **Enable worker threads** matching CPU cores
6. **Use fallback domains** for reliability

## Troubleshooting

### Assets not loading

1. Check domain configuration:

```bash
bunja validate
```

2. Check server logs:

```bash
RUST_LOG=debug bunja serve
```

3. Verify API keys are correct

4. Check cache:

```bash
bunja cache stats
```

### Cache full

Increase cache size in `bunja.lock`:

```toml
[global_settings]
max_cache_size_mb = 4096
```

Or clear cache:

```bash
bunja cache clear
```

### Slow asset loading

1. Prefetch assets:

```bash
bunja prefetch ./public
```

2. Adjust retry settings:

```toml
[domains.my-domain.retry_strategy]
max_retries = 1
backoff_ms = 500
```

3. Check rate limits

### Provider errors

Check provider-specific requirements:

- **Pexels**: Requires API key from pexels.com
- **Unsplash**: Requires access key from unsplash.com
- **S3**: Check bucket permissions and CORS
- **Cloudinary**: Verify cloud name in URL

## Environment Variables

Set log level:

```bash
RUST_LOG=debug bunja serve
RUST_LOG=info bunja translate --input file.html --output out.html
```

## Next Steps

- See `examples/` directory for sample files
- Check `bunja.lock.example` for full configuration
- Run `bunja --help` for command reference
