// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::address_dto_option_packed;
use crate::{
    query::OutputTable,
    types::{
        dtos::{BasicFilterOptionsDto, NftFilterOptionsDto}, AddressDb, AmountDb, FilterOptions, MilestoneIndexDb, OutputIdDb, UnixTimestampDb, NftIdDb,
    },
    Error,
};

use sea_orm::entity::prelude::*;
use sea_query::Cond;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "nft_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alias_id: NftIdDb,
    #[sea_orm(unique)]
    pub output_id: OutputIdDb,
    pub amount: AmountDb,
    pub sender: Option<AddressDb>,
    pub issuer: Option<AddressDb>,
    pub tag: Option<Vec<u8>>,
    pub address: AddressDb,
    pub dust_return: Option<AmountDb>,
    pub dust_return_address: Option<AmountDb>,
    pub timelock_milestone: Option<MilestoneIndexDb>,
    pub timelock_time: Option<UnixTimestampDb>,
    pub expiration_milestone: Option<MilestoneIndexDb>,
    pub expiration_time: Option<UnixTimestampDb>,
    pub expiration_return_address: Option<AddressDb>,
    pub created_at: UnixTimestampDb,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl OutputTable for Entity {
    type FilterOptions = NftFilterOptions;

    fn created_at_col() -> Column {
        Column::CreatedAt
    }

    fn output_id_col() -> Column {
        Column::OutputId
    }
}

#[derive(Debug, Default)]
pub(crate) struct NftFilterOptions {}

impl Into<sea_query::Cond> for NftFilterOptions {
    fn into(self) -> sea_query::Cond {
        todo!()
    }
}

impl TryInto<FilterOptions<Entity>> for NftFilterOptionsDto {
    type Error = Error;

    fn try_into(self) -> Result<FilterOptions<Entity>, Self::Error> {
        Ok(FilterOptions {
            inner: NftFilterOptions {},
            pagination: self.pagination.try_into()?,
            timestamp: self.timestamp.try_into()?,
        })
    }
}
