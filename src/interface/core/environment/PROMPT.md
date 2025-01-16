# Environment Management Module Development Prompt

## Overview
Create a module that manages and switches between different subnet module environments for validators and miners, providing isolation and configuration management.

## Requirements

### Core Functionality
1. Environment Discovery
   - List all running modules on validators/miners
   - Track module configurations and states
   - Monitor environment health

2. Environment Switching
   - Switch between module environments
   - Load appropriate configurations
   - Handle environment variables
   - Manage Docker container contexts

3. Configuration Management
   - Parse module configurations
   - Handle environment variables
   - Manage secrets
   - Support multiple profiles

### Integration Points
- Docker Manager Integration
   - Container lifecycle management
   - Environment isolation
   - Resource management

- Validator/Miner Integration
   - Module registration
   - Status monitoring
   - Configuration updates

### Technical Details
1. Environment Structure
   ```
   module_name/
   ├── .env
   ├── config.toml
   ├── docker-compose.yml (optional)
   └── data/
   ```

2. Configuration Format
   ```toml
   [module]
   name = "example_module"
   version = "1.0.0"
   
   [environment]
   variables = ["API_KEY", "PORT"]
   volumes = ["data:/app/data"]
   
   [docker]
   image = "example:latest"
   ports = ["8080:8080"]
   ```

3. Environment Variables
   - Module-specific variables
   - System variables
   - Secrets management

### Error Handling
- Container errors
- Configuration errors
- Resource conflicts
- Network issues

## Dependencies
- docker_manager
- validator
- serde
- tokio
- tracing
