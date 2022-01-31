// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use sea_orm::entity::prelude::*;

use chrono::NaiveDateTime;

// TODO: Switch to BLOBs once everythign works.
#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alias_outputs")]
pub struct Model {
    #[sea_orm(primary_key)]
    alias_id: String,
    #[sea_orm(created_at = "created_at")]
    pub created_at: NaiveDateTime,
    #[sea_orm(unique)]
    output_id: String,
    amount: i64,
    state_controller: String,
    governor: String,
    issuer: Option<String>,
    sender: Option<String>,
    milestone_index: i64,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
