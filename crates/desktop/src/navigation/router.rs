use zygo_core::models::WorkflowRunId;

use super::Breadcrumb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Routes {
    WorkflowRuns(WorkflowRunsRoutes),
}

impl Routes {
    pub fn breadcrumbs(&self) -> Vec<Breadcrumb> {
        match self {
            Routes::WorkflowRuns(route) => route.breadcrumbs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRunsRoutes {
    Index,
    Run {
        id: WorkflowRunId,
        routes: WorkflowRunRoutes,
    },
}

impl WorkflowRunsRoutes {
    pub fn breadcrumbs(&self) -> Vec<Breadcrumb> {
        let mut breadcrumbs = vec![Breadcrumb {
            label: "Workflow Runs".to_string(),
            route: Routes::WorkflowRuns(WorkflowRunsRoutes::Index),
        }];

        if let WorkflowRunsRoutes::Run { id, routes } = self {
            breadcrumbs.extend(routes.breadcrumbs(id));
        }

        breadcrumbs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRunRoutes {
    Index,
    New,
    JobLog { job_run_id: String, job_id: String },
}

impl WorkflowRunRoutes {
    pub fn breadcrumbs(&self, id: &WorkflowRunId) -> Vec<Breadcrumb> {
        let run_route = |routes| {
            Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                id: id.clone(),
                routes,
            })
        };
        let mut breadcrumbs = vec![Breadcrumb {
            label: id.to_string(),
            route: run_route(WorkflowRunRoutes::Index),
        }];

        if matches!(self, WorkflowRunRoutes::New) {
            breadcrumbs.push(Breadcrumb {
                label: "New".to_string(),
                route: run_route(WorkflowRunRoutes::New),
            });
        }

        if let WorkflowRunRoutes::JobLog { job_run_id, job_id } = self {
            let job_run_id_suffix: String = job_run_id
                .chars()
                .skip(job_run_id.chars().count().saturating_sub(4))
                .collect();

            breadcrumbs.push(Breadcrumb {
                label: format!("{} logs {}", job_id, job_run_id_suffix),
                route: run_route(WorkflowRunRoutes::JobLog {
                    job_run_id: job_run_id.clone(),
                    job_id: job_id.clone(),
                }),
            });
        }

        breadcrumbs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_id() -> WorkflowRunId {
        WorkflowRunId::try_from("run-123".to_string()).expect("test run ID must be valid")
    }

    fn labels(route: &Routes) -> Vec<String> {
        route
            .breadcrumbs()
            .into_iter()
            .map(|breadcrumb| breadcrumb.label)
            .collect()
    }

    #[test]
    fn workflow_runs_index_has_collection_breadcrumb() {
        let route = Routes::WorkflowRuns(WorkflowRunsRoutes::Index);

        assert_eq!(labels(&route), vec!["Workflow Runs"]);
        assert_eq!(
            route.breadcrumbs()[0].route,
            Routes::WorkflowRuns(WorkflowRunsRoutes::Index)
        );
    }

    #[test]
    fn workflow_run_detail_has_collection_and_run_breadcrumbs() {
        let id = run_id();
        let route = Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
            id: id.clone(),
            routes: WorkflowRunRoutes::Index,
        });

        assert_eq!(labels(&route), vec!["Workflow Runs", "run-123"]);
        assert_eq!(
            route.breadcrumbs()[1].route,
            Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                id,
                routes: WorkflowRunRoutes::Index,
            })
        );
    }

    #[test]
    fn new_workflow_run_has_complete_breadcrumb_path() {
        let id = run_id();
        let route = Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
            id: id.clone(),
            routes: WorkflowRunRoutes::New,
        });

        assert_eq!(labels(&route), vec!["Workflow Runs", "run-123", "New"]);
        assert_eq!(
            route.breadcrumbs()[2].route,
            Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                id,
                routes: WorkflowRunRoutes::New,
            })
        );
    }
}
