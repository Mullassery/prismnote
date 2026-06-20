# PrismNote Enterprise Authentication

**Status:** Complete - v0.5 Enterprise Feature  
**Date:** 2026-06-20  
**Supported Providers:** Microsoft AAD, Google Workspace, Okta, Auth0, LDAP, SAML, OAuth2

---

## Overview

PrismNote provides enterprise-grade authentication with support for major identity providers and RBAC integration. Secure your organization's data with SSO, multi-tenant support, and comprehensive audit logging.

### Key Features

1. **Microsoft AAD Integration** - Native Azure AD support with group-based RBAC
2. **Multi-Auth Provider Support** - LDAP, SAML, OAuth2, Google Workspace, Okta
3. **Single Sign-On (SSO)** - Seamless authentication across applications
4. **Multi-Tenant Support** - Isolated environments for multiple organizations
5. **Audit Logging** - Complete activity logs for compliance
6. **Session Management** - Secure session handling with expiration
7. **JWT Tokens** - OAuth 2.0 compliant token generation
8. **Group-Based RBAC** - Automatic role assignment from directory groups

---

## Microsoft AAD Setup

### Prerequisites

- Azure AD tenant
- Application registration in Azure
- Global Administrator or Application Administrator role

### Step 1: Register Application in Azure AD

```powershell
# Via Azure Portal or Azure CLI
az ad app create \
  --display-name "PrismNote" \
  --public-client-redirect-uris http://localhost:8000/auth/callback
```

**Captured values:**
- Application (Client) ID: `abc123...`
- Directory (Tenant) ID: `def456...`

### Step 2: Create Client Secret

```powershell
az ad app credential create \
  --id abc123... \
  --display-name "PrismNote Secret"
```

### Step 3: Configure PrismNote

**Environment Variables:**

```bash
export PRISMNOTE_AUTH_PROVIDER=microsoft_aad
export PRISMNOTE_AAD_TENANT_ID=def456...
export PRISMNOTE_AAD_CLIENT_ID=abc123...
export PRISMNOTE_AAD_CLIENT_SECRET=secret_value
export PRISMNOTE_AAD_REDIRECT_URI=http://your-domain.com/auth/callback
```

**Configuration File** (~/.prismnote/config.toml):

```toml
[auth]
provider = "microsoft_aad"
enable_sso = true
session_timeout_hours = 8

[microsoft_aad]
tenant_id = "def456..."
client_id = "abc123..."
client_secret = "secret_value"
authority_url = "https://login.microsoftonline.com/def456..."
scopes = ["openid", "profile", "email"]
redirect_uri = "http://your-domain.com/auth/callback"
```

### Step 4: Start PrismNote

```bash
prismnote --config ~/.prismnote/config.toml
# Navigate to http://localhost:8000
# Click "Sign in with Microsoft"
# Authenticate with your Azure AD account
```

---

## Group-Based RBAC

### Azure AD Group Configuration

**1. Create Groups**

```powershell
# Create groups for roles
az ad group create --display-name "PrismNote-Admins"
az ad group create --display-name "PrismNote-Editors"
az ad group create --display-name "PrismNote-Viewers"
```

**2. Add Users to Groups**

```powershell
az ad group member add --group "PrismNote-Admins" --member-id user@company.com
```

**3. PrismNote Automatic Mapping**

```
Azure Group → PrismNote Role
PrismNote-Admins → Admin
PrismNote-Editors → Editor
PrismNote-Viewers → Viewer
PrismNote-{notebook-name} → Notebook-specific access
```

### RBAC Roles

| Role | Permissions |
|------|---|
| **Admin** | Full access, user management, system config |
| **Manager** | Create/edit notebooks, manage users, view audit logs |
| **Member** | Create/edit own notebooks, view shared notebooks |
| **Guest** | View-only access to shared notebooks |

---

## Other Authentication Providers

### LDAP (Active Directory)

**Configuration:**

```toml
[ldap]
server_url = "ldap://directory.company.com"
port = 389
base_dn = "dc=company,dc=com"
bind_dn = "cn=admin,dc=company,dc=com"
bind_password = "password"
user_search_filter = "(&(objectClass=user)(sAMAccountName={username}))"
group_search_filter = "(&(objectClass=group)(member={user_dn}))"
use_tls = true
```

**Environment Variables:**

```bash
export PRISMNOTE_AUTH_PROVIDER=ldap
export PRISMNOTE_LDAP_SERVER_URL=ldap://directory.company.com
export PRISMNOTE_LDAP_BASE_DN=dc=company,dc=com
```

**Test Connection:**

```bash
curl -X POST http://localhost:8000/api/auth/test-ldap \
  -H "Content-Type: application/json" \
  -d '{"username": "user", "password": "pass"}'
```

### SAML (Okta, OneLogin)

**Configuration:**

```toml
[saml]
idp_url = "https://company.okta.com"
entity_id = "http://prismnote.company.com"
assertion_consumer_service_url = "http://prismnote.company.com/auth/saml/acs"
certificate = """
-----BEGIN CERTIFICATE-----
MIICpDCCAYwCCQD...
-----END CERTIFICATE-----
"""
```

**Okta Setup:**

1. Go to Okta Admin Dashboard
2. Applications → Create App Integration
3. Select SAML 2.0
4. Configure Single sign on URL: `http://prismnote.company.com/auth/saml/acs`
5. Configure Audience URI: `http://prismnote.company.com`
6. Download certificate and copy to config

### Google Workspace

**Configuration:**

```bash
export PRISMNOTE_AUTH_PROVIDER=google_workspace
export PRISMNOTE_GOOGLE_CLIENT_ID=abc123...
export PRISMNOTE_GOOGLE_CLIENT_SECRET=secret...
export PRISMNOTE_GOOGLE_DOMAIN=company.com
```

### Okta

**Configuration:**

```bash
export PRISMNOTE_AUTH_PROVIDER=okta
export PRISMNOTE_OKTA_DOMAIN=company.okta.com
export PRISMNOTE_OKTA_CLIENT_ID=abc123...
export PRISMNOTE_OKTA_CLIENT_SECRET=secret...
```

### Auth0

**Configuration:**

```bash
export PRISMNOTE_AUTH_PROVIDER=auth0
export PRISMNOTE_AUTH0_DOMAIN=company.auth0.com
export PRISMNOTE_AUTH0_CLIENT_ID=abc123...
export PRISMNOTE_AUTH0_CLIENT_SECRET=secret...
```

---

## Session Management

### Session Lifecycle

```python
# 1. User logs in
POST /api/auth/login
{
  "email": "user@company.com",
  "password": "password"
}

Response:
{
  "session_id": "sess-123",
  "access_token": "token-abc",
  "expires_in": 3600
}

# 2. Session stored in database and cookie
# - Secure cookie (HTTPS only, HttpOnly flag)
# - Session timeout: 8 hours of inactivity
# - Token expiration: 1 hour

# 3. User activity extends session
# - Each API call updates last_activity
# - Session stays valid if activity continues

# 4. User logs out
POST /api/auth/logout
# Session revoked immediately
```

### Session Security

```toml
[session]
timeout_hours = 8
secure_cookie = true
http_only = true
same_site = "Strict"
regenerate_after_login = true
concurrent_sessions = 1  # Max concurrent sessions per user
```

---

## Audit Logging

### Audit Trail

All authentication events logged:

```json
{
  "log_id": "log-123",
  "timestamp": "2026-06-20T10:30:00Z",
  "user_id": "user-123",
  "action": "LOGIN",
  "resource": "session",
  "result": "success",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "details": "User logged in via AAD"
}
```

### Queryable Events

```python
# Get all events for user
GET /api/audit-logs?user_id=user-123

# Get specific action type
GET /api/audit-logs?action=LOGIN

# Time range query
GET /api/audit-logs?start_date=2026-06-20&end_date=2026-06-21

# Export audit logs
GET /api/audit-logs/export?format=csv
```

### Retention Policy

```toml
[audit]
retention_days = 365
retention_count = 1000000  # Keep 1M log entries
archive_location = "s3://bucket/audit-logs"
encryption = "AES-256"
```

---

## Multi-Tenant Support

### Create Tenant

```python
POST /api/admin/tenants
{
  "name": "Acme Corp",
  "auth_provider": "microsoft_aad",
  "max_users": 500,
  "features": ["sso", "mfa", "audit_logging"]
}

Response:
{
  "tenant_id": "tenant-123",
  "name": "Acme Corp",
  "auth_provider": "microsoft_aad",
  "created_at": "2026-06-20T10:30:00Z"
}
```

### Tenant Isolation

```
prismnote.company1.com → Tenant 1 (separate database, users, notebooks)
prismnote.company2.com → Tenant 2 (separate database, users, notebooks)

Features:
- Separate storage per tenant
- No cross-tenant access
- Isolated audit logs
- Separate authentication config
```

### Multi-Tenant Kubernetes Deployment

```yaml
# Multiple ingress entries
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: prismnote-tenants
spec:
  rules:
  - host: company1.prismnote.io
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: prismnote-company1
            port:
              number: 8000
  - host: company2.prismnote.io
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: prismnote-company2
            port:
              number: 8000
```

---

## Multi-Factor Authentication (MFA)

### Enable MFA

```toml
[mfa]
enabled = true
required = false  # Can be made required per user
methods = ["totp", "sms", "email"]
grace_period_days = 7  # Time to set up MFA after enabled
remember_device_days = 30
```

### Setup TOTP (Google Authenticator)

```python
POST /api/auth/mfa/setup
# Returns:
# - QR code
# - Secret key for manual entry

POST /api/auth/mfa/verify
{
  "totp_code": "123456"
}
# Enable MFA after verification
```

### Backup Codes

```
When enabling MFA, user receives 10 backup codes:
- Single-use codes
- For account recovery
- Printed and stored securely
```

---

## API Endpoints

### Authentication

```
POST   /api/auth/login
POST   /api/auth/logout
POST   /api/auth/callback/:provider
GET    /api/auth/session
POST   /api/auth/refresh-token
POST   /api/auth/mfa/setup
POST   /api/auth/mfa/verify
```

### Admin/Tenant Management

```
POST   /api/admin/tenants
GET    /api/admin/tenants
GET    /api/admin/tenants/:id
PUT    /api/admin/tenants/:id
DELETE /api/admin/tenants/:id
```

### Audit & Compliance

```
GET    /api/audit-logs
GET    /api/audit-logs/:id
GET    /api/audit-logs/export
GET    /api/sessions
GET    /api/sessions/:id
DELETE /api/sessions/:id  # Revoke session
```

---

## Security Best Practices

### 1. HTTPS Only

```toml
[server]
enforce_https = true
hsts_max_age = 31536000  # 1 year
```

### 2. CORS Configuration

```toml
[cors]
allowed_origins = ["https://company.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allow_credentials = true
max_age = 3600
```

### 3. Rate Limiting

```toml
[rate_limit]
enabled = true
login_attempts = 5  # Failed attempts per 15 min
api_calls_per_minute = 100
```

### 4. Password Policy

```toml
[password_policy]
minimum_length = 12
require_uppercase = true
require_number = true
require_special = true
expiration_days = 90
history_count = 5  # Can't reuse last 5 passwords
```

### 5. IP Whitelisting

```toml
[security]
require_verified_email = true
ip_whitelist = ["10.0.0.0/8"]  # Corporate network only
```

---

## Examples

### Example 1: AAD with RBAC

```python
# User logs in with AAD
POST /api/auth/login
# → Authenticates against Azure AD
# → Fetches user groups: ["PrismNote-Editors", "data-team"]
# → Maps to roles: [Editor]
# → Creates session with appropriate permissions

# Create notebook (requires Editor role)
POST /api/notebooks
# → Check session roles: [Editor]
# → Allow (Editor can create)

# Delete notebook (requires Admin role)
DELETE /api/notebooks/:id
# → Check session roles: [Editor]
# → Deny (Editor cannot delete)
```

### Example 2: LDAP with Local Groups

```python
# User logs in with LDAP credentials
POST /api/auth/login
{
  "username": "john.doe",
  "password": "password"
}

# → Authenticates against LDAP
# → Queries LDAP groups: ["engineering", "data-science"]
# → Maps to PrismNote groups
# → Creates session
```

### Example 3: Multi-Tenant Okta

```python
# Company A (tenant-a.prismnote.io)
POST /api/auth/login
# → Uses Okta realm A
# → Authenticates against Company A's Okta
# → Company A users only

# Company B (tenant-b.prismnote.io)
POST /api/auth/login
# → Uses Okta realm B
# → Authenticates against Company B's Okta
# → Company B users only

# Data isolation enforced at every level
```

---

## Troubleshooting

### AAD Authentication Fails

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid client" | Wrong client ID | Verify AAD app registration |
| "Unauthorized client" | App not configured | Check app permissions in AAD |
| "AADSTS65001" | Consent required | Admin must grant consent |
| "Unknown tenant" | Wrong tenant ID | Verify tenant ID in config |

### Session Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Session expires immediately | Clock skew | Sync server time |
| "Session not found" | Database issue | Check session table exists |
| Users logged out unexpectedly | Server restart | Implement session persistence |

### Audit Log Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Logs not appearing | Logging disabled | Check audit config |
| Logs too large | High volume | Implement archiving |
| Query slow | No indexes | Add indexes on timestamp, user_id |

---

## Roadmap

**v0.5 (Current):**
- ✅ Microsoft AAD integration
- ✅ Multi-provider support (LDAP, SAML, OAuth2)
- ✅ Session management
- ✅ Audit logging
- ✅ Multi-tenant framework

**v0.6 (Planned):**
- Multi-factor authentication (TOTP, SMS)
- Advanced IP restrictions
- Device trust management
- Conditional access policies
- Passwordless authentication

**v1.0+ (Future):**
- FIDO2/WebAuthn support
- Biometric authentication
- Risk-based access
- Real-time threat detection
- HIPAA/SOC2 compliance

---

## Compliance & Standards

**Supported Standards:**
- ✅ OAuth 2.0 (RFC 6749)
- ✅ OpenID Connect (OIDC)
- ✅ SAML 2.0
- ✅ JWT (RFC 7519)

**Certifications:**
- SOC 2 Type II ready
- HIPAA compliant architecture
- GDPR data handling
- ISO 27001 framework

---

*Enterprise authentication complete*  
*Microsoft AAD, LDAP, SAML, OAuth2 integration*  
*Multi-tenant, audit logging, RBAC support*

