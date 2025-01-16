use crate::TestContext;
use zangief::prelude::*;

#[zangief::tests]
mod validator_tests {
    use super::*;

    #[test]
    async fn test_module_validation(ctx: &TestContext) {
        // Test module validation process
        let validation_request = serde_json::json!({
            "module_name": "test-module",
            "version": "0.1.0",
        });

        let response = ctx.validator_client
            .post("http://localhost:8081/api/v1/validate")
            .json(&validation_request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[test]
    async fn test_inference_execution(ctx: &TestContext) {
        // Test inference execution
        let inference_request = serde_json::json!({
            "module": "test-module",
            "input": {
                "data": "test data"
            }
        });

        let response = ctx.validator_client
            .post("http://localhost:8081/api/v1/infer")
            .json(&inference_request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[test]
    async fn test_resource_management(ctx: &TestContext) {
        // Test resource allocation and management
        // TODO: Implement resource management test
    }
}
