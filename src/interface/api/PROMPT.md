# API Component Development Prompt

## Overview
Create a REST API with WebSocket support that provides programmatic access to the Synapse subnet functionality.

## Requirements

### Core Functionality
- REST endpoints for all CLI operations
- WebSocket for real-time updates
- Swagger documentation
- SS58 key-based authentication

### Integration Points
- Uses core authentication for SS58 verification
- Shares database with other components
- Connects to registrar, validator, and miner services

### Technical Requirements
- Built using Axum web framework
- Async WebSocket handling
- OpenAPI/Swagger documentation
- Rate limiting and security middleware

### Endpoints
1. Authentication:
   - Verify SS58 signatures
   - Session management

2. Registrar Operations:
   - Module management
   - Status updates
   - Configuration

3. Validator Operations:
   - Status monitoring
   - Configuration
   - Performance metrics

4. Miner Operations:
   - Registration
   - Performance tracking
   - Stake management

### Real-time Updates
- WebSocket channels for:
  - Status changes
  - Performance metrics
  - Network events

## Security
- SS58 signature verification
- Rate limiting
- Input validation
- CORS configuration

## Dependencies
- axum
- tower
- tokio
- substrate-interface
- sqlx
- serde
