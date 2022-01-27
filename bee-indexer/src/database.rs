// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::AliasAdapter;

pub(crate) struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    pub(crate) async fn new() -> Result<Self, sqlx::Error> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let pool = sqlx::SqlitePool::connect(&database_url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }

    pub(crate) async fn new_in_memory() -> Result<Self, sqlx::Error> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
}

impl Database {
    pub(crate) async fn insert_alias(&self, alias: AliasAdapter) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query!(
            r#"
            INSERT INTO alias_outputs (alias_id, output_id, amount, state_controller, governor, issuer, sender, milestone_index)
            VALUES  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
            "#,
            alias.alias_id,
            alias.output_id,
            alias.amount,
            alias.state_controller,
            alias.governor,
            alias.issuer,
            alias.sender,
            alias.milestone_index,
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub(crate) async fn get_alias(&self, alias_id: &str) -> Result<AliasAdapter, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        let alias = sqlx::query_as!(
            AliasAdapter,
            r#"SELECT * FROM alias_outputs WHERE alias_id = ?1"#,
            alias_id
        )
        .fetch_one(&mut conn)
        .await?;

        Ok(alias)
    }

    pub(crate) async fn remove_alias(&self, alias_id: &str) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as!(
            AliasAdapter,
            r#"DELETE FROM alias_outputs WHERE alias_id = ?1"#,
            alias_id
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }
}

pub trait Table {
    const TABLE_NAME: &'static str;
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::alias::AliasAdapter;

    use bee_test::rand::{bytes::rand_bytes, number::rand_number};

    #[tokio::test]
    async fn alias_roundtrip() -> Result<(), sqlx::Error> {
        let db = Database::new_in_memory().await?;

        let test_alias = AliasAdapter {
            alias_id: hex::encode(rand_bytes(20)),
            output_id: hex::encode(rand_bytes(34)),
            amount: rand_number(),
            state_controller: hex::encode(rand_bytes(34)),
            governor: hex::encode(rand_bytes(34)),
            issuer: Some(hex::encode(rand_bytes(34))),
            sender: Some(hex::encode(rand_bytes(34))),
            milestone_index: rand_number(),
        };

        db.insert_alias(test_alias.clone()).await?;
        let returned = db.get_alias(&test_alias.alias_id).await?;

        assert_eq!(test_alias, returned);

        db.remove_alias(&test_alias.alias_id).await?;
        let returned = db.get_alias(&test_alias.alias_id).await;
        assert!(returned.is_err());

        Ok(())
    }
}
