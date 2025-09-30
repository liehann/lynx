# Lynx - Link Shortener

A lightweight Rust web application for link shortening and redirection, built with Axum and PostgreSQL.

## Features

- **Smart Redirector**: Host-based routing with exact and progressive path matching
- **Parameterized Links**: Support for dynamic parameters in URLs (e.g., `/user/{id}`)
- **Admin Web UI**: Simple, modern interface for managing links
- **JSON API**: RESTful API for programmatic link management
- **Chrome Extension**: Browser extension for easy go link management
- **In-memory Cache**: Fast lookups with HashMap-based caching
- **Conflict Detection**: Prevents duplicate host/source combinations

## Architecture

- **Web Framework**: Axum with Tokio async runtime
- **Database**: PostgreSQL with sqlx
- **Templates**: Askama for server-side rendering
- **Caching**: In-memory HashMap for fast redirects

## Setup

### Prerequisites

- Rust (latest stable)
- PostgreSQL
- Environment variables (see `.env.example`)

### Database Setup

1. Create a PostgreSQL database:
```sql
CREATE DATABASE lynx_prod;
```

2. Create the links table:
```sql
\c lynx_prod

CREATE TABLE links (
    id SERIAL PRIMARY KEY,
    host TEXT NOT NULL,
    source TEXT NOT NULL,
    target TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE UNIQUE INDEX uniq_links_host_source ON links (host, source);
```

### Configuration

1. Copy the example environment file:
```bash
cp env.example .env
```

2. Update the `.env` file with your database URL:
```
DATABASE_URL=postgresql://username:password@goblin/lynx_prod
ADMIN_HOST=lynx
DEFAULT_REDIRECT_HOST=go
```

### Running

**Option 1: Use the startup script**
```bash
./start-server.sh
```

**Option 2: Manual startup**
```bash
cargo run
```

The application will start on `http://0.0.0.0:3000`.

## Usage

### Admin Interface

Access the admin interface at `http://lynx:3000` (or whatever you set as `ADMIN_HOST`):

- **Home**: View recent links
- **Add Link**: Create new redirects
- **Search**: Find existing links
- **Edit**: Modify or delete links

### Redirector

Configure your DNS or hosts file to point your redirect domain (e.g., `go`) to the server. Then:

- `http://go:3000/docs` → redirects to configured target
- `http://go:3000/user/123` → supports parameterized redirects

### API Endpoints

All API endpoints are available under `/api` on the admin host:

- `GET /api/links` - List recent links
- `POST /api/links` - Create a new link
- `GET /api/links/:id` - Get a specific link
- `PUT /api/links/:id` - Update a link
- `DELETE /api/links/:id` - Delete a link
- `GET /api/links/search?q=query` - Search links
- `GET /api/links/reverse?target=url` - Find links by target URL (reverse lookup)

#### Example API Usage

Create a link:
```bash
curl -X POST http://lynx:3000/api/links \
  -H "Content-Type: application/json" \
  -d '{
    "host": "go",
    "source": "/docs",
    "target": "https://example.com/documentation"
  }'
```

## Link Types

### Static Links
- **Source**: `/docs`
- **Target**: `https://example.com/documentation`
- **Usage**: `go:3000/docs` → `https://example.com/documentation`

### Parameterized Links
- **Source**: `/user/{id}`
- **Target**: `https://example.com/profile?user={id}`
- **Usage**: `go:3000/user/123` → `https://example.com/profile?user=123`

### Progressive Matching
If no exact match is found, the system progressively strips path segments:
- `/docs/api/v1` → tries `/docs/api` → tries `/docs`

## Chrome Extension

Lynx includes a Chrome extension for easy go link management directly from your browser.

### Features
- Smart icon that shows color when go links exist, greyscale when none exist
- Quick popup to view existing go links and add new ones
- One-click copying of go links to clipboard
- Automatic detection when navigating to new pages
- Support for "go" domain only

### Installation
1. Open Chrome and go to `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked" and select the `chrome/` directory
4. Configure your Lynx server URL in the extension settings

See `chrome/README.md` for detailed installation and usage instructions.

## Development

### Running Tests

**Option 1: Use the test script**
```bash
./run-tests.sh
```

**Option 2: Manual testing**
```bash
# Unit tests only (no database required)
cargo test --lib

# Integration tests (requires test database)
cargo test --test integration_tests
```

**Test Configuration:**
For integration tests, create a `.env.test` file:
```bash
cp env.example .env.test
# Edit .env.test with test database credentials
```

### Database Migrations

Migrations are handled manually for production safety. The migration files are in the `migrations/` directory.

**To run migrations manually:**
```sql
-- Connect to your database and run the migration SQL
\c lynx_prod
\i migrations/001_create_links_table.sql
```

### Project Structure

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports
├── config.rs        # Configuration management
├── models.rs        # Data models
├── database.rs      # Database operations
├── redirector.rs    # Redirect logic
├── templates.rs     # Template definitions
└── handlers/        # HTTP handlers
    ├── mod.rs
    ├── api.rs       # JSON API endpoints
    └── ui.rs        # Web UI handlers

templates/           # Askama templates
migrations/          # Database migrations
tests/              # Integration tests
```

## License

MIT License - see LICENSE file for details.