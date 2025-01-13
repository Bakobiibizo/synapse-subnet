use crate::TestContext;
use zangief::prelude::*;

#[zangief::tests]
mod registrar_tests {
    use super::*;

    #[test]
    async fn test_module_registration(ctx: &TestContext) {
        // Test module registration flow
        let module_data = serde_json::json!({
            "name": "test-module",
            "version": "0.1.0",
            "repo_url": "https://github.com/test/module",
            "branch": "main",
            "description": "Test module",
            "author": "Test Author",
            "license": "MIT",
            "module_type": "inference",
        });

        let response = ctx.registrar_client
            .post("http://localhost:8080/api/v1/modules")
            .json(&module_data)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[test]
    async fn test_module_verification(ctx: &TestContext) {
        // Test module verification process
        // TODO: Implement verification test
    }

    #[test]
    async fn test_module_retrieval(ctx: &TestContext) {
        // Test module retrieval
        let response = ctx.registrar_client
            .get("http://localhost:8080/api/v1/modules/test-module")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }
}
