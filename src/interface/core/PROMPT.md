# Core Component Development Prompt

## Overview
Create the shared core functionality used by CLI, API, and GUI components of the Synapse subnet interface.

## Requirements

### Core Functionality
- SS58 key management
- Database operations
- Configuration management
- Shared types and utilities

### Integration Points
- Used by CLI, API, and GUI components
- Connects to subnet services
- Manages shared state

### Technical Requirements
- Async operations
- Error handling
- Type safety
- Database migrations

### Modules
1. Authentication:
   - SS58 key operations
   - Signature verification
   - Session management

2. Database:
   - Connection management
   - Migrations
   - Transaction handling

3. Configuration:
   - Environment loading
   - Validation
   - Type-safe access

4. Models:
   - Shared data types
   - Validation
   - Serialization

### Error Handling
- Custom error types
- Result wrappers
- Error propagation

## Dependencies
- substrate-interface
- sqlx
- serde
- tokio
- tracing
