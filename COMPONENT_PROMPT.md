# Component Library Generation Prompt

You are tasked with creating a component for the Synapse Subnet(synapse-subnet) library. Follow this structured approach:

## System Overview
The system is built around a module registrar system that ingests subnet modules from substrate blockchains, extracts environment variables and configuration options and builds a docker container with the subnet running a validator inside of it and then packages that up into a self-installing module that is provided via API to the validator from the registrar.

The validator is currently working as intended, it clones the repo and produces the container image and runs it with the provided environment variables and configuration options.

The subnets we are running provide a miner implementation as well that generally is concretely defined in a file called `miner.py`. We need to follow a similar structure to the validator implementation to grab the miner.py and modify the module package to let you choose which one to install. The miner has a slightly different configuration though it is largely just a component of the validator.

## Components Required
Module Miner Component

### 1. Core Requirements
- Purpose: Dynamically install and run subnet-modules with the miner configuration 
- Primary Features:
  - Module extraction and verification
  - Docker container management
  - Configuration management
  - State management
  - Real-time metrics collection
  - WebSocket status updates
  - Stake management
  - Priority-based retry mechanism
- Data Models:
  ```rust
  // Miner configuration
  pub struct MinerConfig {
      pub module_name: String,
      pub stake_amount: u64,
      pub auto_restake: bool,
      pub priority_level: PriorityLevel,
      pub resource_limits: ResourceLimits,
  }

  // Resource limits
  pub struct ResourceLimits {
      pub cpu_cores: f32,
      pub memory_mb: u32,
      pub storage_gb: u32,
  }

  // Mining metrics
  pub struct MinerMetrics {
      pub total_blocks: u64,
      pub success_rate: f64,
      pub average_block_time: u64,
      pub rewards_earned: u64,
  }

  // Module status
  pub struct ModuleStatus {
      pub is_active: bool,
      pub current_stake: u64,
      pub uptime: u64,
      pub last_update: DateTime<Utc>,
  }
  ```

### 2. Integration Points
- External Systems:
  - Docker daemon for container management
  - Substrate blockchain for mining operations
  - Module registry for package management
  - WebSocket server for real-time updates
- Communication Protocols:
  - HTTP/WebSocket for API communication
  - Docker API for container management
  - Substrate RPC for blockchain interaction
- Security Requirements:
  - Secure key management
  - Resource isolation in containers
  - Rate limiting for mining operations
  - Stake verification
  - Priority-based access control

### 3. Performance Targets
- Response Time:
  - Container startup: < 5s
  - Mining operation: < 100ms
  - Status updates: < 50ms
- Resource Limits:
  - Memory: 512MB per miner instance
  - CPU: 2 cores per instance
  - Storage: 5GB per module
- Scalability Needs:
  - Support for 100+ concurrent miners
  - Auto-scaling based on load
  - Efficient resource utilization

### 4. Error Handling
- Scenarios:
  - Network disconnection
  - Container failure
  - Resource exhaustion
  - Invalid stake amount
  - Priority throttling
  - Module verification failure
- Recovery:
  - Automatic retry with backoff
  - State recovery from disk
  - Resource cleanup
  - Stake reallocation

### 5. Testing Requirements
- Unit Tests:
  - Configuration parsing
  - State management
  - Metric collection
  - Error handling
- Integration Tests:
  - Container lifecycle
  - Mining operations
  - Stake management
  - Real-time updates
- Performance Tests:
  - Load testing
  - Resource utilization
  - Network resilience

### 6. Monitoring and Metrics
- System Health:
  - Container status
  - Resource usage
  - Network connectivity
  - Stake distribution
- Performance Metrics:
  - Mining success rate
  - Block time statistics
  - Reward distribution
  - Priority queue status

### 7. Documentation Requirements
- Setup Guide:
  - Installation steps
  - Configuration options
  - Environment setup
  - Security considerations
- API Documentation:
  - Endpoint descriptions
  - Request/response formats
  - Error codes
  - WebSocket protocol
- Operation Guide:
  - Monitoring procedures
  - Troubleshooting steps
  - Performance tuning
  - Recovery procedures

## Implementation Instructions

1. For each component:
   - Create the component structure following the template
   - Implement error handling with specific error types
   - Add comprehensive tests
   - Document public interfaces

2. Progress tracking must include:
   - Current status of each component
   - Recent updates
   - Next steps
   - Known issues

3. Documentation must cover:
   - Usage examples
   - Configuration options
   - Error handling
   - Performance characteristics

## Output Requirements

Generate the following for each component:

1. Component implementation with:
   - Clear interfaces
   - Error handling
   - Configuration options
   - Unit tests

2. Integration tests demonstrating:
   - Normal operation
   - Error scenarios
   - Performance requirements

3. Documentation including:
   - Usage examples
   - API reference
   - Error handling guide
   - Performance notes

4. Progress tracking with:
   - Implementation status
   - Known issues
   - Next steps
   - Recent updates

## Example Usage

To generate a component library, replace the placeholders with specific requirements:

```markdown
# Component Library Generation Prompt

## System Overview
Name: Authentication System
Description: Secure user authentication with OAuth2 support

## Components Required
1. OAuth2 Client
2. Token Manager
3. User Session Handler

## For Component: OAuth2 Client

### 1. Core Requirements
- Purpose: Handle OAuth2 authentication flow
- Primary Features:
  - Authorization code flow
  - Refresh token handling
  - State management
- Data Models:
  - OAuth2Token
  - UserCredentials
- Error Scenarios:
  - Invalid credentials
  - Network timeout
  - Token expiration

### 2. Integration Points
- External Systems: OAuth2 providers (Google, GitHub)
- Communication Protocols: HTTPS with JWT
- Security Requirements: TLS 1.3, secure token storage

### 3. Performance Targets
- Response Time: < 500ms
- Resource Limits: 50MB RAM per instance
- Scalability Needs: 1000 auth requests/second
```

This template can be used by providing:
1. System overview and requirements
2. List of components needed
3. Specific requirements for each component
4. Integration and performance targets

The AI will then generate:
1. Complete component implementations
2. Test suites
3. Documentation
4. Progress tracking

Would you like me to:
1. Generate a specific component library?
2. Modify the template structure?
3. Add more example scenarios?


Note: The library has ARCHITECTURE.md, and PROGRESS.md files to track progress and reference to avoid confusion. Please use them as needed