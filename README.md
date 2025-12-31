# Rausth

A reusable authentication server API in Rust for Vercel Serverless Functions.

## Features

- **Multiple Authentication Methods**:
  - Email + Password authentication
  - Google OAuth
  - Microsoft OAuth  
  - GitHub OAuth

- **Token-based Authentication**:
  - JWT access tokens
  - Refresh tokens with database storage
  - Configurable token expiration

- **User Management**:
  - Automatic user creation on first authentication
  - User data storage in PostgreSQL

- **Serverless Architecture**:
  - Deployed as Vercel Serverless Functions
  - Automatic scaling
  - Low latency with global edge network

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/Souchy/Rausth.git
cd Rausth

# Set up database
createdb rausth

# Configure environment variables
export DATABASE_URL="******localhost/rausth"
export JWT_SECRET="your-secret-key"
export AUTH_EMAIL_PASSWORD="true"

# Build
cargo build --release
```

### Deploy to Vercel

1. Install Vercel CLI:
```bash
npm i -g vercel
```

2. Set up environment variables in Vercel:
```bash
vercel env add DATABASE_URL
vercel env add JWT_SECRET
vercel env add AUTH_EMAIL_PASSWORD
# Add other environment variables as needed
```

3. Deploy:
```bash
vercel deploy
```

## Documentation

- [Usage Guide](USAGE.md) - Detailed API usage examples
- [Contributing](CONTRIBUTING.md) - How to contribute to the project

## Requirements

- Rust 1.70 or higher
- PostgreSQL database (e.g., Vercel Postgres, Neon, Supabase)
- Vercel account for deployment

## Configuration

### Required Settings

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT token signing

### Authentication Methods

Enable or disable authentication methods via environment variables:

- `AUTH_EMAIL_PASSWORD`: Enable email/password authentication (default: true)
- `AUTH_GOOGLE_ENABLED`: Enable Google OAuth (default: false)
- `AUTH_MICROSOFT_ENABLED`: Enable Microsoft OAuth (default: false)
- `AUTH_GITHUB_ENABLED`: Enable GitHub OAuth (default: false)

### OAuth Provider Configuration

For each OAuth provider you want to enable, set these environment variables:

**Google OAuth:**
```bash
AUTH_GOOGLE_ENABLED=true
AUTH_GOOGLE_CLIENT_ID=your-google-client-id
AUTH_GOOGLE_CLIENT_SECRET=your-google-client-secret
AUTH_GOOGLE_REDIRECT_URI=https://your-domain.vercel.app/api/oauth/google/callback
```

**Microsoft OAuth:**
```bash
AUTH_MICROSOFT_ENABLED=true
AUTH_MICROSOFT_CLIENT_ID=your-microsoft-client-id
AUTH_MICROSOFT_CLIENT_SECRET=your-microsoft-client-secret
AUTH_MICROSOFT_REDIRECT_URI=https://your-domain.vercel.app/api/oauth/microsoft/callback
```

**GitHub OAuth:**
```bash
AUTH_GITHUB_ENABLED=true
AUTH_GITHUB_CLIENT_ID=your-github-client-id
AUTH_GITHUB_CLIENT_SECRET=your-github-client-secret
AUTH_GITHUB_REDIRECT_URI=https://your-domain.vercel.app/api/oauth/github/callback
```

See `.env.example` for a complete configuration example.

## API Endpoints

All endpoints are deployed as serverless functions at `/api/*`.

### Email/Password Authentication

**Register a new user**
```http
POST /api/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Login**
```http
POST /api/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

### OAuth Authentication

**Google OAuth**
```http
GET /api/oauth/google
```
Redirects to Google login page.

```http
POST /api/oauth/google/callback
Content-Type: application/json

{
  "code": "authorization_code_from_google"
}
```

**Microsoft OAuth**
```http
GET /api/oauth/microsoft
```

```http
POST /api/oauth/microsoft/callback
Content-Type: application/json

{
  "code": "authorization_code_from_microsoft"
}
```

**GitHub OAuth**
```http
GET /api/oauth/github
```

```http
POST /api/oauth/github/callback
Content-Type: application/json

{
  "code": "authorization_code_from_github"
}
```

### Token Refresh

```http
POST /api/refresh
Content-Type: application/json

{
  "refresh_token": "your_refresh_token"
}
```

## Response Format

All authentication endpoints return the same response format:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Database Schema

The application automatically creates the required database tables on first connection:

- `users`: Stores user information and authentication details
- `refresh_tokens`: Stores refresh tokens with expiration

## Security Considerations

- Always use HTTPS in production (Vercel provides this by default)
- Set a strong JWT secret in environment variables
- Use connection pooling for database (configured by default)
- Rotate refresh tokens after use (automatic)
- Set appropriate token expiration times
- Keep OAuth client secrets in Vercel environment variables, not in code
- Use Vercel Postgres or another managed PostgreSQL service with SSL

## Development

Build the library:
```bash
cargo build --lib
```

Build a specific function:
```bash
cargo build --bin register
```

Check all functions:
```bash
cargo check --bins
```

Format code:
```bash
cargo fmt
```

Check for issues:
```bash
cargo clippy
```

## Deployment

### Vercel Deployment

1. **Install Vercel CLI:**
```bash
npm i -g vercel
```

2. **Link your project:**
```bash
vercel link
```

3. **Set environment variables:**
```bash
vercel env add DATABASE_URL production
vercel env add JWT_SECRET production
# Add other variables as needed
```

4. **Deploy:**
```bash
vercel deploy --prod
```

### Environment Variables in Vercel

Set these in your Vercel project settings or via CLI:

- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret for JWT signing
- `AUTH_EMAIL_PASSWORD` - Enable email/password auth
- OAuth configuration variables (if using OAuth)

## Examples

See [USAGE.md](USAGE.md) for detailed usage examples including:
- Complete authentication flows
- OAuth provider setup
- Integration with frontend applications
- Error handling
- Vercel deployment guide

## License

This project is available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

Future enhancements:
- Additional OAuth providers (Apple, Twitter, etc.)
- Two-factor authentication (2FA)
- Email verification
- Password reset functionality
- Rate limiting
- Session management
- Admin API for user management

