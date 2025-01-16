# CLI Component Development Prompt

## Overview
Create a command-line interface for the Synapse subnet that allows users to interact with registrars, validators, and miners.

## Requirements

### Core Functionality
- Command structure for registrar, validator, and miner interactions
- SS58 key management commands
- Configuration management
- Status monitoring and logging

### Integration Points
- Uses core authentication module for SS58 key operations
- Connects to API for remote operations
- Shares database access with other components

### Technical Requirements
- Built using Clap for command parsing
- Implements async operations with Tokio
- Uses substrate-interface for SS58 key operations
- Follows same configuration pattern as validator crate

### Command Groups
1. Registrar Commands:
   - Register/unregister modules
   - List modules
   - Update module status

2. Validator Commands:
   - Start/stop validation
   - Monitor status
   - Configure settings

3. Miner Commands:
   - Register/unregister
   - Monitor performance
   - Manage stake

4. Key Management:
   - Generate keys
   - Import/export keys
   - Sign messages

## Configuration
- Uses same environment-based config as validator
- Supports multiple profiles
- Allows override via command line flags

## Dependencies
- clap
- tokio
- substrate-interface
- sqlx
- tracing
