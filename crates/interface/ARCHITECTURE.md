# Synapse Interface Architecture

## Overview
The interface system provides a unified way to interact with the Synapse subnet through multiple interfaces:
- CLI for direct command-line operations
- REST API for programmatic access
- Web GUI for human interaction

## Components

### interface-core
Core library containing:
- Shared data models
- Authentication/authorization using SS58 keys
- Database interactions
- Common utilities and types

### interface-cli
Command-line interface providing:
- Direct interaction with registrar, validators, and miners
- Key management
- Configuration management
- Status monitoring

### interface-api
REST API service with:
- Swagger documentation
- SS58 key-based authentication
- Endpoints mirroring CLI functionality
- Query API for validator/miner data
- WebSocket support for real-time updates

### interface-gui
Web interface built with:
- Rust-based templating (Askama)
- HTMX for dynamic updates
- Lightweight component system
- Real-time monitoring dashboards

## Data Flow
1. All interfaces use interface-core for shared functionality
2. Authentication flows through SS58 key verification
3. Database operations are isolated per component
4. Real-time updates use WebSocket when available

## Security
- SS58 key pairs for authentication
- Separate database instances for different components
- Rate limiting on API endpoints
- Input validation at all entry points

## Dependencies
- substrate-interface: SS58 key handling
- axum: Web framework
- sqlx: Database operations
- askama: Templating
- htmx-rs: Dynamic UI updates
