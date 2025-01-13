# Registrar Service Architecture

## Overview
The Registrar service is responsible for managing subnet modules in the Synapsis network. It provides functionality for registering, managing, and retrieving subnet modules, using SQLite for persistent storage and exposing a REST API for module management.

## Components

### Core Components
- **Registry**: Handles all database operations for subnet modules
- **Registrar**: Main service that coordinates module management
- **API**: REST endpoints for module management
- **Commands**: CLI commands for module operations

### Database Schema
The service uses SQLite with the following schema:
```sql
CREATE TABLE subnet_modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    branch TEXT NOT NULL,
    description TEXT NOT NULL,
    author TEXT NOT NULL,
    license TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    downloads INTEGER NOT NULL DEFAULT 0,
    module_type TEXT NOT NULL,
    status TEXT NOT NULL
);
```

### Directory Structure
```
registrar/
├── src/
│   ├── api/            # REST API endpoints
│   ├── commands/       # CLI command implementations
│   ├── docker/         # Docker runtime management
│   ├── verification/   # Module verification logic
│   ├── lib.rs         # Library exports
│   ├── main.rs        # CLI entrypoint
│   └── registry.rs    # Database operations
├── migrations/         # Database migrations
├── config/            # Module configurations
└── data/             # Database files
```

## Module Lifecycle
1. **Registration**: Modules are registered via the CLI or API
2. **Verification**: Module code is verified for security and compatibility
3. **Storage**: Module metadata is stored in SQLite
4. **Deployment**: Modules can be deployed using Docker
5. **Management**: Module status and metadata can be updated

## Security Considerations
- Module verification before registration
- Secure storage of module metadata
- Access control for module management
- Docker isolation for module execution

## Future Improvements
- [ ] Add module versioning support
- [ ] Implement module dependencies
- [ ] Add module update mechanisms
- [ ] Improve verification process
