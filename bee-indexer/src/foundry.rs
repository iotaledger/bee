use crate::types::{Address, MilestoneIndex};

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Foundry {
    pub foundry_id: String,
    pub output_id: String,
    pub amount: i64,
    pub address: Option<String>,
    pub milestone_index: MilestoneIndex,
}

pub(crate) async fn insert_foundry_output<'a>(pool: &sqlx::SqlitePool, foundry: Foundry) -> Result<i64, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
        INSERT INTO foundry_outputs
        ( foundry_id, output_id, amount, address, milestone_index )
        VALUES  (?1, ?2, ?3, ?4, ?5);
        "#,
        foundry.foundry_id,
        foundry.output_id,
        foundry.amount,
        foundry.address,
        foundry.milestone_index,
        
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}