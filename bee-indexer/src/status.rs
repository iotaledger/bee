// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use sea_orm::entity::prelude::*;

use crate::types::MilestoneIndexDb;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "status")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: u32,
    pub(crate) current_milestone_index: MilestoneIndexDb,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
