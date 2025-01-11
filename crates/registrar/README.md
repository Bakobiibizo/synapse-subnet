# Synapse Subnet Registrar

The Synapse Subnet Registrar is a tool for managing, installing, and running subnet modules. It provides a complete workflow for ingesting subnet modules from git repositories, managing their environments, and running them with proper configuration.

## Quick Start

```bash
# One-step registration (recommended)
cargo run -p registrar -- register --repo https://github.com/example/subnet-module

# Or step by step:
# 1. Ingest a module from a git repository
cargo run -p registrar -- ingest --repo https://github.com/example/subnet-module

# 2. Install and run the module
cargo run -p registrar -- install my-subnet
```

## Commands

### Ingest Module
Ingest a subnet module from a git repository. This will:
- Clone the repository into the subnet-modules directory
- Create configuration files in the config directory
- Generate an install script

```bash
cargo run -p registrar -- ingest --repo <repository-url> [options]

Options:
  --name <name>      Custom name for the module (defaults to repo name)
  --branch <branch>  Branch or tag to checkout (defaults to main)
  -h, --help        Print help
```

### Register Module (All-in-One)
Register a new subnet module by performing all necessary steps in sequence:
1. Ingest the module from a git repository
2. Create and configure the environment
3. Install and run the module

```bash
cargo run -p registrar -- register --repo <repository-url> [options]

Options:
  --name <name>           Custom name for the module (defaults to repo name)
  --branch <branch>       Branch or tag to checkout (defaults to main)
  --accept-defaults      Accept default environment values without prompting
  --registrar-port <port> Port for the registrar (default: 8080)
  --validator-port <port> Port for the validator (default: 8081)
  --skip-docker-build    Skip Docker build if image already exists
  -h, --help            Print help
```

### Environment Management
Manage environment variables for different subnet modules.

```bash
# Create a new environment
cargo run -p registrar -- env create <name> [options]

Options:
  --from-example <path>  Path to .env.example file to use as template
  --accept-defaults     Accept default values without prompting
  -h, --help           Print help

# List environments
cargo run -p registrar -- env list [options]

Options:
  --show-values  Show environment variable values
  -h, --help    Print help

# Activate an environment
cargo run -p registrar -- env activate <name>
```

### Install and Run
Install and run a subnet module. This command handles the complete setup process:
- Environment configuration
- Docker image building
- Registrar startup
- Module registration
- Validator startup

```bash
cargo run -p registrar -- install <name> [options]

Options:
  --skip-env-setup     Skip environment setup and use existing environment
  --skip-docker-build  Skip Docker build
  --skip-registration  Skip module registration
  --registrar-port <port>  Port for the registrar (default: 8080)
  --validator-port <port>  Port for the validator (default: 8081)
  -h, --help          Print help
```

### Start Server
Start the registrar server standalone.

```bash
cargo run -p registrar -- start [options]

Options:
  --port <port>  Port to run the server on (default: 8080)
  -h, --help    Print help
```

## Module Structure

When a module is ingested, it creates the following structure:

```
crates/registrar/
├── subnet-modules/          # Module source code
│   └── my-subnet/          # Cloned repository
├── config/                 # Configuration files
│   ├── .env/              # Environment files
│   │   ├── active.env     # Symlink to active environment
│   │   └── my-subnet.env  # Module environment variables
│   └── my-subnet/         # Module configuration
│       ├── config.yaml    # Module configuration
│       ├── .env.example   # Environment template
│       └── install.sh     # Installation script
```

## Environment Files

Environment files (.env) store configuration for each subnet module. They can be created from .env.example templates or with default values. The active environment is symlinked to active.env.

## Installation Process

The installation process:

1. **Environment Setup**
   - Creates environment from .env.example if it doesn't exist
   - Activates the environment

2. **Docker Setup**
   - Checks for Docker installation
   - Builds the module's Docker image

3. **Service Startup**
   - Starts the registrar server
   - Registers the module
   - Starts the validator
   - Handles cleanup on shutdown

## Development

### Adding a New Module

1. Create a new repository for your subnet module
2. Include a Dockerfile for building the module
3. Add an .env.example for required environment variables
4. Ingest and install the module using the commands above

### Module Requirements

- Must have a Dockerfile in the root directory
- Should include an .env.example if environment variables are needed
- Should follow the subnet module interface specification

## Troubleshooting

### Common Issues

1. **Docker Issues**
   - Ensure Docker is installed and running
   - Check if the module's Dockerfile is valid

2. **Environment Issues**
   - Verify all required environment variables are set
   - Check if the environment is properly activated

3. **Port Conflicts**
   - Use --registrar-port and --validator-port to change ports
   - Ensure no other services are using the same ports

### Logs

- Registrar logs are output to stdout/stderr
- Use --show-values with env list to debug environment variables
