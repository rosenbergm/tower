use anyhow::anyhow;
use tokio::process::Command;

pub async fn run_migrations() -> anyhow::Result<()> {
    let status = Command::new("dbmate")
        .arg("migrate")
        .spawn()
        .expect("Cannot run `dbmate` to migrate the database!")
        .wait()
        .await?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("`dbmate` failed with a non-zero status code"))
    }
}
