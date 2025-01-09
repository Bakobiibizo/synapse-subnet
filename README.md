# Synapse Subnet

A modular validator-miner framework for blockchain subnets, specializing in LLM text generation inference.

## Overview

The Synapse Subnet project provides a flexible and modular framework for implementing blockchain subnet validators and miners. It specifically focuses on Large Language Model (LLM) text generation workloads, using Ollama for model serving.

## Components

- **Validator**: Handles request validation, load balancing, and result verification
- **Miner**: Manages model serving and inference execution
- **Registrar**: Provides module registry and build system
- **Chain API**: Manages blockchain integration with support for Commune and Subspace networks
- **GUI**: (Future) Provides monitoring and management interface

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Python 3.10 or later
- Docker
- Git

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
```

## Features

- **Blockchain Integration**
  - Commune network support for module registration and staking
  - Subspace network integration for data storage and retrieval
  - Comprehensive test suite for network operations

- **Module Management**
  - Module registration and discovery
  - Stake management with minimum stake requirements
  - Permission-based access control

## Development

See [PROGRESS.md](PROGRESS.md) for current development status and roadmap.

## License

[MIT License](LICENSE)
