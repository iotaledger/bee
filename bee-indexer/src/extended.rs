// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Extended {
    pub output_id: String,
    pub amount: i64,
    pub sender: Option<String>,
    pub tag: Option<String>,
    pub address: Option<String>,
    pub milestone_index: String,
}

pub(crate) async fn insert_extended_output<'a>(
    pool: &sqlx::SqlitePool,
    extended: Extended,
) -> Result<i64, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
        INSERT INTO extended_outputs
        ( output_id, amount, sender, tag, address, milestone_index )
        VALUES  (?1, ?2, ?3, ?4, ?5, ?6);
        "#,
        extended.output_id,
        extended.amount,
        extended.sender,
        extended.tag,
        extended.address,
        extended.milestone_index,
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}
