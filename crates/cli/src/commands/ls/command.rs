use anyhow::{Result, bail};
use local::{DEFAULT_DATABASE_BUSY_TIMEOUT, ZygoLocalConfig, ZygoLocalService};
use zygo_core::ZygoConfig;

pub async fn list_workflow_runs(filter: Option<&str>) -> Result<()> {
    let filter = filter.map(parse_filter).transpose()?;
    let service = ZygoLocalService::new(ZygoLocalConfig {
        base: ZygoConfig::new(1),
        database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
    })
    .await?;

    for workflow_run in service.list_workflow_runs(filter).await? {
        println!("{}", workflow_run.id);
    }

    Ok(())
}

fn parse_filter(filter: &str) -> Result<(&str, &str)> {
    let Some((key, value)) = filter.split_once('=') else {
        bail!("invalid filter '{filter}': expected KEY=VALUE");
    };

    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        bail!("invalid filter '{filter}': key and value must not be empty");
    }

    Ok((key, value))
}
