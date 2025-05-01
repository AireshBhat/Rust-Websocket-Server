# API Documentation

## HTTP Endpoints

### Authentication
**POST** `/api/auth/login`

- Request (application/json):
```json
{
  "email": "string",
  "password": "string"
}
```
- Response `200 OK` (application/json):
```json
{
  "token": "string",
  "user": {
    "id": 1,
    "email": "string",
    "username": "string",
    "wallet_address": "string | null",
    "created_at": "ISO8601 datetime",
    "last_active": "ISO8601 datetime"
  },
  "expires_at": "ISO8601 datetime"
}
```

### Users

**POST** `/api/users`
- Request (application/json):
```json
{
  "email": "string",
  "username": "string",
  "password": "string",
  "wallet_address": "string | null"
}
```
- Response `201 Created` (application/json): returns a `User` object
```json
{
  "id": 1,
  "email": "string",
  "username": "string",
  "wallet_address": "string | null",
  "created_at": "ISO8601 datetime",
  "last_active": "ISO8601 datetime"
}
```

**GET** `/api/users/{id}`
- Response `200 OK` (application/json): returns a `User` object

**PUT** `/api/users/{id}`
- Request (application/json):
```json
{
  "username": "string | null",
  "email": "string | null",
  "wallet_address": "string | null"
}
```
- Response `200 OK` (application/json): returns the updated `User` object

**DELETE** `/api/users/{id}`
- Response `204 No Content`

### Public Key Management

**POST** `/api/users/{id}/keys`
- Request (application/json):
```json
{
  "public_key": "hex-encoded string"
}
```
- Response `201 Created` (application/json):
```json
{
  "status": "success",
  "message": "Public key added successfully"
}
```

**GET** `/api/users/{id}/keys`
- Response `200 OK` (application/json):
```json
{
  "user_id": 1,
  "public_keys": ["hex-encoded key1", "hex-encoded key2"]
}
```

**DELETE** `/api/users/{id}/keys/{key}`
- Response `200 OK` (application/json) if revoked:
```json
{
  "status": "success",
  "message": "Public key revoked successfully"
}
```
- Response `404 Not Found` (application/json) if not found:
```json
{
  "status": "error",
  "message": "Public key not found or already revoked"
}
```

### Network
*(No HTTP endpoints implemented yet)*

### Earnings
*(No HTTP endpoints implemented yet)*

### Referrals
*(No HTTP endpoints implemented yet)*

---

## WebSocket Endpoints

All WebSocket routes share the same protocol and message types defined under `WebSocketMessage`.

### Common WS Message Types
- **Auth**: authenticate with ed25519 signature
  ```json
  {
    "type": "Auth",
    "data": {
      "public_key": "hex-encoded string",
      "timestamp": 1617181723,
      "nonce": "string",
      "signature": "hex-encoded string"
    }
  }
  ```
- **Heartbeat**: keepalive ping/pong (binary/ping frames)
- **ConnectionUpdate**:
  ```json
  {"type":"ConnectionUpdate","data":{"connected":true}}
  ```
- **NetworkUpdate**:
  ```json
  {"type":"NetworkUpdate","data":{"status":"string","score":0.0}}
  ```
- **EarningsUpdate**:
  ```json
  {"type":"EarningsUpdate","data":{"amount":0.0,"source":"string"}}
  ```
- **Error**:
  ```json
  {"type":"Error","data":{"code":"string","message":"string"}}
  ```
- **Data**:
  ```json
  {"type":"Data","data":{"content":<any JSON value>}}
  ```

### Dashboard WebSocket
**GET** `/ws/dashboard` (Upgrade to WebSocket)
- Server sends on connect:
  ```json
  {"type":"connection_established","session_id":"string","auth_required":true,"message":"Please authenticate with an ed25519 signature"}
  ```
- Client must send **Auth** message first
- On success:
  ```json
  {"type":"auth_success","session_id":"string","message":"Authentication successful"}
  ```
- On failure or timeout, server sends **Error** and closes
- Afterwards, server streams `ConnectionUpdate`, `NetworkUpdate`, `EarningsUpdate`, or other **Data** messages

### Earnings WebSocket
**GET** `/ws/earnings` (Upgrade to WebSocket)
- Same protocol as `/ws/dashboard`, filtering for earnings updates

### Referrals WebSocket
**GET** `/ws/referrals` (Upgrade to WebSocket)
- Same protocol as `/ws/dashboard`, filtering for referral updates
