# Validator

We need to create a validator to ensure that the subnet is functioning correctly and the miners are providing valid data to the network. Because of the large overhead of running a validator for every individual subnet i have decided to create a single validator for the entire network. 
It will install centralized inference modules to use to generate the various kinds of inference regardless of the subnet. It does this by using the registrar to pull the required install file and install the module. 
It will also install the subnet code as a module, identify the subnet inference requirements and inject the subnet code with the correct inference modules required. 
In addition it will pull any required configuration and prompt the user to set any required parameters. 
Once configured it will launch a validator for that subnet and monitor the requests and responses. 
In theory the validator shouldnt have to make any requests to the network as its subnet module will handle that. 

## Requirements
- [ ] Install inference modules from the registrar
- [ ] Install subnet code from the registrar
- [ ] Identify subnet inference requirements
- [ ] Inject subnet code with the required inference modules
- [ ] Pull any required configuration
- [ ] Prompt the user to set any required parameters
- [ ] Launch the validator
- [ ] Monitor the requests and responses

## Validator Progress

### Current Status
- [x] Basic validator manager structure
- [x] Docker container management integration
- [x] Module registration and management
- [x] Subnet verification logic
- [x] Integration with registrar API
- [x] Test infrastructure with mocks
- [ ] Comprehensive error handling
- [ ] Metrics and monitoring
- [ ] Performance optimization

### Recent Updates (2025-01-09)
- Added integration with registrar API client for module management
- Implemented proper cleanup of containers and modules
- Added test mocks using wiremock for testing without a running registrar
- Fixed issues with module registration and startup
- Improved test coverage for module installation and subnet verification

### Next Steps
1. Add comprehensive error handling for API failures
2. Implement metrics collection for module performance
3. Add monitoring capabilities for module health
4. Optimize container startup and shutdown sequences
5. Add support for module configuration validation

### Known Issues
- Need to improve error handling for API communication failures
- Could improve test coverage for edge cases
- Need to add proper logging throughout the codebase

### Dependencies
- docker-manager: Container lifecycle management
- registrar-api: Communication with the registrar service
- registrar: Core module management types
- wiremock (dev): Mock HTTP server for testing