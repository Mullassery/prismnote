use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbtProject {
    pub name: String,
    pub path: String,
    pub profiles_dir: String,
    pub target: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbtModel {
    pub name: String,
    pub path: String,
    pub model_type: String,
    pub description: Option<String>,
    pub columns: Vec<DbtColumn>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbtColumn {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub tests: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbtTest {
    pub name: String,
    pub model: String,
    pub test_type: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbtRunResult {
    pub run_id: String,
    pub status: String,
    pub models_run: usize,
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub execution_time_seconds: f32,
}

pub struct DbtManager {
    pub project: DbtProject,
}

impl DbtManager {
    pub fn new(project: DbtProject) -> Self {
        Self { project }
    }

    pub async fn list_models(&self) -> Result<Vec<DbtModel>, String> {
        // TODO: Implement dbt project parsing
        Ok(vec![
            DbtModel {
                name: "example_model".to_string(),
                path: "models/example.sql".to_string(),
                model_type: "table".to_string(),
                description: Some("Example dbt model".to_string()),
                columns: vec![
                    DbtColumn {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                        description: None,
                        tests: vec!["unique".to_string(), "not_null".to_string()],
                    },
                ],
            },
        ])
    }

    pub async fn run_dbt(&self, _selector: Option<String>) -> Result<DbtRunResult, String> {
        // TODO: Execute dbt run
        Ok(DbtRunResult {
            run_id: "dbt-run-123".to_string(),
            status: "success".to_string(),
            models_run: 5,
            tests_run: 15,
            tests_passed: 15,
            tests_failed: 0,
            execution_time_seconds: 45.2,
        })
    }

    pub async fn run_tests(&self) -> Result<Vec<DbtTest>, String> {
        // TODO: Execute dbt test
        Ok(vec![
            DbtTest {
                name: "test_id_unique".to_string(),
                model: "users".to_string(),
                test_type: "unique".to_string(),
                status: "pass".to_string(),
            },
        ])
    }

    pub async fn generate_docs(&self) -> Result<String, String> {
        // TODO: Generate dbt docs site
        Ok("dbt documentation generated".to_string())
    }

    pub async fn get_lineage(&self, _model: &str) -> Result<String, String> {
        // TODO: Return model lineage graph
        Ok("Model lineage (v1.0+ feature)".to_string())
    }

    pub fn generate_profiles_yml() -> String {
        r#"prismnote:
  outputs:
    dev:
      type: postgres
      host: localhost
      user: postgres
      password: password
      port: 5432
      dbname: analytics
      schema: dbt_dev
      threads: 4
    prod:
      type: postgres
      host: prod-db.example.com
      user: dbt_prod
      password: [password]
      port: 5432
      dbname: analytics
      schema: dbt_prod
      threads: 8
  target: dev
"#
        .to_string()
    }

    pub fn generate_dbt_project_yml(project_name: &str) -> String {
        format!(
            r#"name: '{}'
version: '1.0.0'
config-version: 2

profile: 'prismnote'
model-paths: ["models"]
analysis-paths: ["analysis"]
test-paths: ["tests"]
data-paths: ["data"]
macro-paths: ["macros"]
snapshot-paths: ["snapshots"]
target-path: "target"
clean-targets:
  - "target"
  - "dbt_packages"

models:
  {}:
    materialized: view
"#,
            project_name, project_name
        )
    }
}
