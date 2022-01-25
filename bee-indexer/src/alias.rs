use sqlx::Executor;

use crate::types::{Address, MilestoneIndex};

#[derive(Debug, Eq, sqlx::FromRow, PartialEq)]
pub(crate) struct Alias {
    pub alias_id: String,
    pub output_id: String,
    pub amount: i64,
    pub state_controller: Address,
    pub governor: Address,
    pub issuer: Option<Address>,
    pub sender: Option<Address>,
    pub milestone_index: MilestoneIndex,
}

pub(crate) async fn insert_alias_output<'a>(pool: &sqlx::SqlitePool, alias: &Alias) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
        INSERT INTO alias_outputs
        ( alias_id, output_id, amount, state_controller, governor, issuer, sender, milestone_index )
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

pub(crate) async fn get_alias_output(pool: &sqlx::SqlitePool, id: &str) -> Result<Alias, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let alias = sqlx::query_as!(Alias, r#"SELECT * FROM alias_outputs WHERE alias_id = ?1"#, id)
        .fetch_one(&mut conn)
        .await?;

    Ok(alias)
}

#[cfg(test)]
mod test {

    use crate::{database::Database, alias::{Alias, insert_alias_output, get_alias_output}};

    use bee_test::rand::{number::rand_number, bytes::rand_bytes};

    #[tokio::test]
    async fn alias_roundtrip() -> Result<(), sqlx::Error> {

        let db = Database::new_in_memory().await?;
    
        let test_alias = Alias {
            alias_id: hex::encode(rand_bytes(20)),
            output_id: hex::encode(rand_bytes(34)),
            amount: rand_number(),
            state_controller: hex::encode(rand_bytes(34)),
            governor: hex::encode(rand_bytes(34)),
            issuer: Some(hex::encode(rand_bytes(34))),
            sender: Some(hex::encode(rand_bytes(34))),
            milestone_index: rand_number(),
        };
    
        insert_alias_output(&db.pool, &test_alias).await?;
        let returned = get_alias_output(&db.pool, &test_alias.alias_id).await?;
    
        assert_eq!(test_alias, returned);

        Ok(())
    }
}

