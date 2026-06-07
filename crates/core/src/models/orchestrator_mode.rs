#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorMode {
    Local,
    Remote,
}

impl OrchestratorMode {
    pub fn from_env() -> Self {
        match std::env::var("ORCHESTRATOR_MODE")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase()
            .as_str()
        {
            "remote" => Self::Remote,
            _ => Self::Local,
        }
    }
}

impl std::fmt::Display for OrchestratorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Remote => write!(f, "remote"),
        }
    }
}
