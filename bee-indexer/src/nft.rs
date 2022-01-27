// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Nft {
    pub nft_id: String,
    pub output_id: String,
    pub amount: i64,
    pub issuer: Option<String>,
    pub sender: Option<String>,
    pub tag: Option<String>,
    pub address: Option<String>,
    pub milestone_index: String,
}

pub(crate) async fn insert_nft_output<'a>(pool: &sqlx::SqlitePool, nft: Nft) -> Result<i64, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
        INSERT INTO nft_outputs
        ( nft_id, output_id, amount, issuer, sender, tag, address, milestone_index )
        VALUES  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
        "#,
        nft.nft_id,
        nft.output_id,
        nft.amount,
        nft.issuer,
        nft.sender,
        nft.tag,
        nft.address,
        nft.milestone_index,
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}
