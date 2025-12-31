# Rausth Usage Examples

This document provides practical examples for using the Rausth authentication server API.

## Setup

1. **Configure your database**:
```bash
# Create a PostgreSQL database
createdb rausth

# Set environment variables
export DATABASE_URL="postgresql://username:password@localhost/rausth"
export JWT_SECRET="your-secret-key-change-this-in-production"
```

2. **Start the server**:
```bash
cargo run
```

The server will start on `http://localhost:8000` by default.

## Email/Password Authentication

### Register a New User

```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securepassword123"
  }'
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Login

```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securepassword123"
  }'
```

## OAuth Authentication

### Google OAuth Flow

1. **Initiate OAuth flow** (in a browser):
```
http://localhost:8000/auth/oauth/google
```

This redirects to Google's login page.

2. **Handle callback** (after user approves):
```bash
curl -X POST http://localhost:8000/auth/oauth/google/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "authorization_code_from_google"
  }'
```

### Microsoft OAuth Flow

1. **Initiate OAuth flow**:
```
http://localhost:8000/auth/oauth/microsoft
```

2. **Handle callback**:
```bash
curl -X POST http://localhost:8000/auth/oauth/microsoft/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "authorization_code_from_microsoft"
  }'
```

### GitHub OAuth Flow

1. **Initiate OAuth flow**:
```
http://localhost:8000/auth/oauth/github
```

2. **Handle callback**:
```bash
curl -X POST http://localhost:8000/auth/oauth/github/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "authorization_code_from_github"
  }'
```

## Token Refresh

When your access token expires, use the refresh token to get a new one:

```bash
curl -X POST http://localhost:8000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "your-refresh-token-here"
  }'
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "new-refresh-token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Using Access Tokens

Include the access token in your API requests:

```bash
curl http://your-api.com/protected-endpoint \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## OAuth Provider Setup

### Google OAuth

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable Google+ API
4. Create OAuth 2.0 credentials
5. Add `http://localhost:8000/auth/oauth/google/callback` as redirect URI
6. Copy Client ID and Client Secret to your configuration

### Microsoft OAuth

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to Azure Active Directory > App registrations
3. Create a new registration
4. Add redirect URI: `http://localhost:8000/auth/oauth/microsoft/callback`
5. Create a client secret
6. Copy Application (client) ID and client secret to your configuration

### GitHub OAuth

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Set Authorization callback URL: `http://localhost:8000/auth/oauth/github/callback`
4. Register application
5. Copy Client ID and generate Client Secret
6. Add credentials to your configuration

## Configuration Examples

### Environment Variables (.env)

```env
DATABASE_URL=postgresql://localhost/rausth
JWT_SECRET=your-secret-key-here

# Enable email/password
AUTH_EMAIL_PASSWORD=true

# Enable Google OAuth
AUTH_GOOGLE_ENABLED=true
AUTH_GOOGLE_CLIENT_ID=your-google-client-id
AUTH_GOOGLE_CLIENT_SECRET=your-google-client-secret
AUTH_GOOGLE_REDIRECT_URI=http://localhost:8000/auth/oauth/google/callback

# Enable GitHub OAuth
AUTH_GITHUB_ENABLED=true
AUTH_GITHUB_CLIENT_ID=your-github-client-id
AUTH_GITHUB_CLIENT_SECRET=your-github-client-secret
AUTH_GITHUB_REDIRECT_URI=http://localhost:8000/auth/oauth/github/callback
```

### TOML Configuration (config.toml)

```toml
[server]
host = "0.0.0.0"
port = 8000

[database]
url = "postgresql://localhost/rausth"

[auth]
email_password = true

[auth.google]
enabled = true
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"
redirect_uri = "http://localhost:8000/auth/oauth/google/callback"

[jwt]
secret = "your-secret-key-here"
access_token_expiry_seconds = 3600
refresh_token_expiry_seconds = 604800
```

## Integration Example (JavaScript/TypeScript)

```typescript
// Register a new user
async function register(email: string, password: string) {
  const response = await fetch('http://localhost:8000/auth/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  
  const data = await response.json();
  localStorage.setItem('access_token', data.access_token);
  localStorage.setItem('refresh_token', data.refresh_token);
  return data;
}

// Login
async function login(email: string, password: string) {
  const response = await fetch('http://localhost:8000/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  
  const data = await response.json();
  localStorage.setItem('access_token', data.access_token);
  localStorage.setItem('refresh_token', data.refresh_token);
  return data;
}

// Refresh token
async function refreshAccessToken() {
  const refreshToken = localStorage.getItem('refresh_token');
  const response = await fetch('http://localhost:8000/auth/refresh', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: refreshToken })
  });
  
  const data = await response.json();
  localStorage.setItem('access_token', data.access_token);
  localStorage.setItem('refresh_token', data.refresh_token);
  return data;
}

// Make authenticated request
async function makeAuthenticatedRequest(url: string) {
  const accessToken = localStorage.getItem('access_token');
  const response = await fetch(url, {
    headers: {
      'Authorization': `Bearer ${accessToken}`
    }
  });
  
  if (response.status === 401) {
    // Token expired, refresh and retry
    await refreshAccessToken();
    return makeAuthenticatedRequest(url);
  }
  
  return response.json();
}
```

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error message here"
}
```

Common HTTP status codes:
- `200 OK` - Success
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Invalid credentials or token
- `404 Not Found` - Resource not found
- `409 Conflict` - User already exists
- `500 Internal Server Error` - Server error

## Security Best Practices

1. **Always use HTTPS in production**
2. **Store tokens securely** - Use HttpOnly cookies or secure storage
3. **Rotate refresh tokens** - Tokens are automatically rotated on refresh
4. **Set appropriate token expiration times**
5. **Keep your JWT secret secure** - Never commit it to version control
6. **Use strong passwords** - Consider adding password strength requirements
7. **Enable only needed auth methods** - Disable unused OAuth providers
8. **Monitor for suspicious activity** - Track failed login attempts
