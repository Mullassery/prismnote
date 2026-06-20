use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CellId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CellType {
    Code,
    Markdown,
    SQL,
    PySpark,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellNode {
    pub id: CellId,
    pub cell_type: CellType,
    pub code: String,
    pub status: ExecutionStatus,
    pub execution_time_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
    pub depends_on: Vec<CellId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub notebook_id: String,
    pub nodes: HashMap<CellId, CellNode>,
    pub execution_order: Vec<CellId>,
    pub total_execution_time_ms: u64,
}

pub struct ExecutionPipeline {
    plans: HashMap<String, ExecutionPlan>,
    execution_cache: HashMap<CellId, ExecutionResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub cell_id: CellId,
    pub status: ExecutionStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub dependencies_satisfied: bool,
}

impl ExecutionPipeline {
    pub fn new() -> Self {
        ExecutionPipeline {
            plans: HashMap::new(),
            execution_cache: HashMap::new(),
        }
    }

    pub fn build_plan(
        &mut self,
        notebook_id: String,
        cells: Vec<CellNode>,
    ) -> Result<ExecutionPlan> {
        let mut nodes = HashMap::new();
        let mut dependency_graph: HashMap<CellId, Vec<CellId>> = HashMap::new();

        // Build dependency graph from cell code analysis
        for cell in cells {
            let cell_id = cell.id.clone();
            let mut deps = cell.depends_on.clone();

            // Analyze code for variable dependencies
            if let CellType::Code | CellType::SQL | CellType::PySpark = cell.cell_type {
                deps.extend(self.analyze_dependencies(&cell.code));
            }

            dependency_graph.insert(cell_id.clone(), deps);
            nodes.insert(cell_id, cell);
        }

        // Topological sort to determine execution order
        let execution_order = self.topological_sort(&dependency_graph)?;

        let plan = ExecutionPlan {
            notebook_id,
            nodes,
            execution_order,
            total_execution_time_ms: 0,
        };

        Ok(plan)
    }

    pub fn get_execution_plan(&self, notebook_id: &str) -> Option<ExecutionPlan> {
        self.plans.get(notebook_id).cloned()
    }

    pub fn get_next_executable(&self, notebook_id: &str) -> Option<CellId> {
        if let Some(plan) = self.plans.get(notebook_id) {
            for cell_id in &plan.execution_order {
                if let Some(node) = plan.nodes.get(cell_id) {
                    if node.status == ExecutionStatus::Pending {
                        // Check if all dependencies are completed
                        let deps_satisfied = node
                            .depends_on
                            .iter()
                            .all(|dep_id| {
                                plan.nodes
                                    .get(dep_id)
                                    .map(|n| n.status == ExecutionStatus::Completed)
                                    .unwrap_or(false)
                            });

                        if deps_satisfied {
                            return Some(cell_id.clone());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn record_execution(&mut self, result: ExecutionResult) {
        self.execution_cache.insert(result.cell_id.clone(), result);
    }

    pub fn can_skip_cell_execution(&self, notebook_id: &str, cell_id: &CellId) -> bool {
        // Skip if cell hasn't changed and all dependencies haven't changed
        if let Some(cached_result) = self.execution_cache.get(cell_id) {
            if cached_result.status == ExecutionStatus::Completed {
                // Check if dependencies are in cache and unchanged
                if let Some(plan) = self.plans.get(notebook_id) {
                    if let Some(node) = plan.nodes.get(cell_id) {
                        return node
                            .depends_on
                            .iter()
                            .all(|dep| self.execution_cache.contains_key(dep));
                    }
                }
            }
        }
        false
    }

    fn analyze_dependencies(&self, code: &str) -> Vec<CellId> {
        // Simple variable reference detection
        let mut deps = vec![];
        let lines: Vec<&str> = code.lines().collect();

        for line in lines {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            // Very basic heuristic: look for variable assignments and usages
            if let Some(eq_pos) = line.find('=') {
                let before_eq = &line[..eq_pos].trim();
                if !before_eq.contains('=')
                    && !before_eq.contains('<')
                    && !before_eq.contains('>')
                    && !line.starts_with("def ")
                    && !line.starts_with("class ")
                {
                    // This looks like an assignment, track the variable
                    if let Ok(var_name) = before_eq.split_whitespace().next().ok_or("") {
                        // Store variable usage (simplified)
                    }
                }
            }
        }

        deps
    }

    fn topological_sort(
        &self,
        graph: &HashMap<CellId, Vec<CellId>>,
    ) -> Result<Vec<CellId>> {
        let mut sorted = vec![];
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for cell_id in graph.keys() {
            if !visited.contains(cell_id) {
                self.dfs_visit(cell_id, graph, &mut visited, &mut visiting, &mut sorted)?;
            }
        }

        // dfs_visit recurses into a node's dependencies before pushing the node,
        // so `sorted` is already in dependency-first order. (No reverse: the graph
        // stores dependencies as edges, not dependents.)
        Ok(sorted)
    }

    fn dfs_visit(
        &self,
        node: &CellId,
        graph: &HashMap<CellId, Vec<CellId>>,
        visited: &mut std::collections::HashSet<CellId>,
        visiting: &mut std::collections::HashSet<CellId>,
        sorted: &mut Vec<CellId>,
    ) -> Result<()> {
        if visited.contains(node) {
            return Ok(());
        }

        if visiting.contains(node) {
            return Err(anyhow::anyhow!("Circular dependency detected at {:?}", node));
        }

        visiting.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                self.dfs_visit(neighbor, graph, visited, visiting, sorted)?;
            }
        }

        visiting.remove(node);
        visited.insert(node.clone());
        sorted.push(node.clone());

        Ok(())
    }

    pub fn get_execution_statistics(&self, notebook_id: &str) -> ExecutionStatistics {
        let mut total_time = 0u64;
        let mut completed_cells = 0;
        let mut failed_cells = 0;
        let mut pending_cells = 0;

        if let Some(plan) = self.plans.get(notebook_id) {
            for node in plan.nodes.values() {
                match node.status {
                    ExecutionStatus::Completed => {
                        completed_cells += 1;
                        total_time += node.execution_time_ms;
                    }
                    ExecutionStatus::Failed => failed_cells += 1,
                    ExecutionStatus::Pending => pending_cells += 1,
                    _ => {}
                }
            }
        }

        ExecutionStatistics {
            total_cells: (completed_cells + failed_cells + pending_cells) as u32,
            completed_cells,
            failed_cells,
            pending_cells,
            total_execution_time_ms: total_time,
            average_cell_time_ms: if completed_cells > 0 {
                total_time / completed_cells
            } else {
                0
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    pub total_cells: u32,
    pub completed_cells: u64,
    pub failed_cells: u64,
    pub pending_cells: u64,
    pub total_execution_time_ms: u64,
    pub average_cell_time_ms: u64,
}

impl Default for ExecutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let mut graph = HashMap::new();
        let cell1 = CellId("1".to_string());
        let cell2 = CellId("2".to_string());
        let cell3 = CellId("3".to_string());

        graph.insert(cell1.clone(), vec![]);
        graph.insert(cell2.clone(), vec![cell1.clone()]);
        graph.insert(cell3.clone(), vec![cell2.clone()]);

        let pipeline = ExecutionPipeline::new();
        let order = pipeline.topological_sort(&graph).unwrap();

        assert_eq!(order, vec![cell1, cell2, cell3]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = HashMap::new();
        let cell1 = CellId("1".to_string());
        let cell2 = CellId("2".to_string());

        graph.insert(cell1.clone(), vec![cell2.clone()]);
        graph.insert(cell2.clone(), vec![cell1.clone()]);

        let pipeline = ExecutionPipeline::new();
        let result = pipeline.topological_sort(&graph);

        assert!(result.is_err());
    }
}
