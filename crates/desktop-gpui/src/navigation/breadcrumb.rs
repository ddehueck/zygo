use super::Routes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumb {
    pub label: String,
    pub route: Routes,
}
