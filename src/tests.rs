//! Snapshot tests for WatsonX client

#[cfg(test)]
mod snapshot_tests {
    use crate::{WatsonxClient, WatsonxConfig};
    use insta::assert_snapshot;

    #[test]
    #[ignore] // Snapshot tests disabled - prompts have been centralized
    fn test_config_snapshot() {
        let config = WatsonxConfig::new("test_api_key_redacted".to_string(), "test_project_id".to_string());

        assert_snapshot!(format!("{:?}", config));
    }

    #[test]
    #[ignore] // Snapshot tests disabled - prompts have been centralized
    fn test_quality_assessment_snapshot() {
        let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());
        let client = WatsonxClient::new(config).unwrap();

        let test_cases = vec![
            ("ibmcloud resource groups", "good_command"),
            ("ibmcloud login --sso", "login_command"),
            ("ibmcloud target -r us-south", "target_command"),
            ("error: invalid command", "error_command"),
            ("", "empty_command"),
        ];

        for (command, name) in test_cases {
            let score = client.assess_quality(command, "test prompt");
            assert_snapshot!(format!("quality_{}: {}", name, score));
        }
    }

    #[test]
    #[ignore] // Snapshot tests disabled - prompts have been centralized
    fn test_model_constants() {
        assert_snapshot!(format!(
            "granite_4_h_small: {}",
            crate::models::models::GRANITE_4_H_SMALL
        ));
        assert_snapshot!(format!(
            "granite_3_3_8b: {}",
            crate::models::models::GRANITE_3_3_8B_INSTRUCT
        ));
    }
}
