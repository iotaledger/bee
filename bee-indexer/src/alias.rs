// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use sea_orm::entity::prelude::*;

use chrono::NaiveDateTime;

use crate::types::{AddressDb, AliasIdDb};

// TODO: Switch to BLOBs once everythign works.
#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alias_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alias_id: AliasIdDb,
    #[sea_orm(created_at = "created_at")]
    pub created_at: NaiveDateTime,
    #[sea_orm(unique)]
    pub output_id: AddressDb,
    pub amount: i64,
    pub state_controller: AddressDb,
    pub governor: AddressDb,
    pub issuer: Option<AddressDb>,
    pub sender: Option<AddressDb>,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
