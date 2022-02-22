// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    output::{OutputTable,address_dto_option_packed},
    types::{AddressDb, AmountDb, FoundryIdDb, OutputIdDb, UnixTimestampDb},
    Error, FoundryFilterOptionsDto,
};

use sea_orm::entity::prelude::*;
use sea_query::Cond;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "foundry_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub foundry_id: FoundryIdDb,
    #[sea_orm(unique)]
    pub output_id: OutputIdDb,
    pub amount: AmountDb,
    pub address: AddressDb,
    pub created_at: UnixTimestampDb,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl OutputTable for Entity {
    type FilterOptions = FoundryFilterOptions;
    type FilterOptionsDto = FoundryFilterOptionsDto;

    const ENTITY: Self = Self;

    fn created_at_col() -> Column {
        Column::CreatedAt
    }

    fn output_id_col() -> Column {
        Column::OutputId
    }
}

#[derive(Debug, Default)]
pub(crate) struct FoundryFilterOptions {
    unlockable_by_address: Option<AddressDb>,
}

impl Into<sea_query::Cond> for FoundryFilterOptions {
    fn into(self) -> sea_query::Cond {
        Cond::all().add_option(self.unlockable_by_address.map(|addr| Column::Address.eq(addr)))
    }
}

impl TryInto<FoundryFilterOptions> for FoundryFilterOptionsDto {
    type Error = Error;

    fn try_into(self) -> Result<FoundryFilterOptions, Self::Error> {
        Ok(FoundryFilterOptions {
            unlockable_by_address: address_dto_option_packed(self.unlockable_by_address, "unlockable by address")?,
        })
    }
}
