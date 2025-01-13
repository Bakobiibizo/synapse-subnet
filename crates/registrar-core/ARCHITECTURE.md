# Registrar Core Architecture

## Overview
The registrar-core crate serves as the foundation for the Synapse Subnet system, providing core types, traits, and interfaces that are shared across other components.

## Core Components

### 1. Module Types
```rust
pub struct Module {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub module_type: ModuleType,
    pub status: ModuleStatus,
}
```

The Module struct represents the fundamental unit of deployment in the system.

### 2. Module Status Management
```rust
pub enum ModuleStatus {
    Created,
    Running,
    Stopped,
    Failed,
}
```

Tracks the lifecycle state of modules in the system.

### 3. Registry Interface
```rust
#[async_trait]
pub trait Registry: Send + Sync {
    async fn get_module(&self, name: &str) -> Result<Option<Module>, Error>;
    async fn list_modules(&self) -> Result<Vec<Module>, Error>;
    async fn create_module(&self, module: &Module) -> Result<i64, Error>;
    // ... other methods
}
```

Defines the core interface for module management.

## Key Features

### 1. Error Handling
- Comprehensive error types
- Error propagation patterns
- Result type aliases

### 2. Type Safety
- Strong typing for module states
- Validation at type level
- Serialization support

### 3. Async Support
- Async trait definitions
- Future compatibility
- Runtime agnostic design

## Interface Contracts

### Registry Operations
1. Module Creation
   - Validation requirements
   - Unique constraints
   - Version management

2. Module Retrieval
   - Caching considerations
   - Error conditions
   - Optional returns

3. Status Updates
   - State transitions
   - Validation rules
   - Event propagation

## Data Models

### Module Configuration
```yaml
name: string
version: string
type: enum
  - validator
  - inference
  - storage
status: enum
  - created
  - running
  - stopped
  - failed
```

### Error Types
```rust
pub enum Error {
    NotFound(String),
    AlreadyExists(String),
    InvalidState(String),
    Database(String),
    // ... other error types
}
```

## Best Practices

### 1. Error Handling
- Use custom error types
- Provide context
- Support error chaining

### 2. Async Operations
- Use async traits
- Provide timeouts
- Handle cancellation

### 3. Type Safety
- Use strong types
- Validate at compile time
- Provide type conversion

## Integration Points

### 1. Registrar Integration
- Module management
- Status tracking
- Configuration handling

### 2. Validator Integration
- Module verification
- Status updates
- Health checks

### 3. API Integration
- Type serialization
- Error mapping
- Status synchronization
