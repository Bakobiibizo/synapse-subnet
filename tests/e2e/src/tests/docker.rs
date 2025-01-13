use crate::TestContext;
use zangief::prelude::*;

#[zangief::tests]
mod docker_tests {
    use super::*;

    #[test]
    async fn test_container_lifecycle(ctx: &TestContext) {
        // Test container creation and lifecycle
        let container_request = serde_json::json!({
            "module": "test-module",
            "version": "0.1.0",
            "resources": {
                "cpu": 1,
                "memory": "512M"
            }
        });

        let response = ctx.validator_client
            .post("http://localhost:8081/api/v1/containers")
            .json(&container_request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[test]
    async fn test_container_health(ctx: &TestContext) {
        // Test container health monitoring
        let response = ctx.validator_client
            .get("http://localhost:8081/api/v1/containers/test-module/health")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[test]
    async fn test_container_cleanup(ctx: &TestContext) {
        // Test container cleanup
        let response = ctx.validator_client
            .delete("http://localhost:8081/api/v1/containers/test-module")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }
}
