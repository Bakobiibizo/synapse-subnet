# Registrar API Architecture

## Overview
The registrar-api crate provides the HTTP API interface for the Synapse Subnet system, enabling communication between different components and external clients.

## Core Components

### 1. API Router
```rust
pub fn create_router(registry: impl Registry) -> Router {
    Router::new()
        .route("/modules", get(list_modules))
        .route("/modules", post(create_module))
        .route("/modules/:name", get(get_module))
        // ... other routes
}
```

### 2. Request/Response Types
```rust
#[derive(Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub module_type: ModuleType,
}

#[derive(Serialize)]
pub struct ModuleResponse {
    pub name: String,
    pub status: ModuleStatus,
    pub module_type: ModuleType,
}
```

### 3. Client Implementation
```rust
pub struct RegistrarClient {
    client: Client,
    base_url: String,
}
```

## API Endpoints

### 1. Module Management
- `GET /modules` - List all modules
- `POST /modules` - Create new module
- `GET /modules/:name` - Get module details
- `PUT /modules/:name/status` - Update status
- `DELETE /modules/:name` - Remove module

### 2. Health & Status
- `GET /health` - API health check
- `GET /modules/:name/status` - Module status
- `GET /metrics` - System metrics

## Error Handling

### 1. API Errors
```rust
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
    // ... other errors
}
```

### 2. Error Responses
```json
{
    "error": {
        "code": "NOT_FOUND",
        "message": "Module not found: test-module"
    }
}
```

## Client Features

### 1. Module Operations
- Create modules
- List modules
- Get module details
- Update module status
- Delete modules

### 2. Error Handling
- Automatic retry
- Error parsing
- Status code handling

### 3. Configuration
- Base URL
- Timeouts
- Retry policy

## Security

### 1. Authentication
- API key support
- Token validation
- Role-based access

### 2. Request Validation
- Input sanitization
- Schema validation
- Rate limiting

## Integration Points

### 1. Registry Integration
- Module operations
- Status updates
- Configuration management

### 2. Client Integration
- Error handling
- Response parsing
- Request building

### 3. Metrics Integration
- Performance tracking
- Error tracking
- Usage statistics

## Best Practices

### 1. API Design
- RESTful principles
- Consistent responses
- Proper status codes

### 2. Error Handling
- Detailed error messages
- Proper status codes
- Error categorization

### 3. Performance
- Connection pooling
- Response caching
- Efficient serialization

## Testing Strategy

### 1. Unit Tests
- Handler tests
- Error handling tests
- Validation tests

### 2. Integration Tests
- API endpoint tests
- Client tests
- Error scenario tests

### 3. Load Tests
- Performance testing
- Stress testing
- Concurrency testing
