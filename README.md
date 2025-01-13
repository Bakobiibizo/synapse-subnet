# Synapse Subnet

A modular validator-miner framework for blockchain subnets, specializing in LLM text generation inference.

## Overview

The Synapse Subnet project provides a flexible and modular framework for implementing blockchain subnet validators and miners. It specifically focuses on Large Language Model (LLM) text generation workloads, using Ollama for model serving.

## Components

### Core Services

- **Validator**: Handles request validation, load balancing, and result verification
- **Miner**: Manages model serving and inference execution
- **Registrar**: Provides module registry and build system
  - SQLite-based module storage
  - Docker-based module runtime
  - Module verification and security checks
- **Chain API**: Manages blockchain integration with support for Commune and Subspace networks

### Supporting Libraries

- **registrar-core**: Core types and traits for the registrar service
- **registrar-api**: Client library for interacting with the registrar service
- **docker-manager**: Docker container management utilities

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Python 3.10 or later
- Docker
- Git
- SQLite 3

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/synapse-subnet.git
cd synapse-subnet

# Set up Python virtual environment for Commune integration
python -m venv .commune-env
source .commune-env/bin/activate
pip install communex

# Build the project
cargo build

# Run tests
cargo test

# Start the registrar service
cargo run -p registrar -- serve --port 8082
```

## Features

### Blockchain Integration
- Commune network support for module registration and staking
- Subspace network integration for data storage and retrieval
- Comprehensive test suite for network operations

### Module Management
- Module registration and discovery via REST API
- SQLite-based persistent storage
- Docker-based module runtime environment
- Module verification and security checks
- Permission-based access control

### Development Tools
- CLI tools for module management
- Client library for service integration
- Comprehensive documentation and examples

## Architecture

Each component has its own architecture documentation:
- [Registrar Architecture](crates/registrar/ARCHITECTURE.md)
- [Validator Architecture](crates/validator/ARCHITECTURE.md)
- [Docker Manager](crates/docker-manager/ARCHITECTURE.md)

## Development

The project is under active development. Key features and improvements:

- [x] SQLite-based module registry
- [x] Docker runtime integration
- [x] Basic module verification
- [ ] Advanced security checks
- [ ] GUI interface
- [ ] Module versioning
- [ ] Dependency management

## License

[MIT License](LICENSE)
