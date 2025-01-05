# Synapse Subnet

A modular validator-miner framework for blockchain subnets, specializing in LLM text generation inference.

## Overview

The Synapse Subnet project provides a flexible and modular framework for implementing blockchain subnet validators and miners. It specifically focuses on Large Language Model (LLM) text generation workloads, using Ollama for model serving.

## Components

- **Validator**: Handles request validation, load balancing, and result verification
- **Miner**: Manages model serving and inference execution
- **Registrar**: Provides module registry and build system
- **Chain API**: Manages blockchain integration
- **GUI**: (Future) Provides monitoring and management interface

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Docker
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/synapse-subnet.git
cd synapse-subnet

# Build the project
cargo build

# Run tests
cargo test
```

## Development

See [PROGRESS.md](PROGRESS.md) for current development status and roadmap.

## License

[MIT License](LICENSE)
