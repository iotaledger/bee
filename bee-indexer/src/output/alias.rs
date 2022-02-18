// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::address_dto_option_packed;
use crate::{
    query::{OutputTable, IndexedOutputTable},
    types::FilterOptions,
    types::{AddressDb, AliasIdDb, AmountDb, UnixTimestampDb, OutputIdDb},
    AliasFilterOptionsDto, Error,
};

use sea_orm::entity::prelude::*;
use sea_query::Cond;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alias_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alias_id: AliasIdDb,
    #[sea_orm(unique)]
    pub output_id: OutputIdDb,
    pub amount: AmountDb,
    pub state_controller: AddressDb,
    pub governor: AddressDb,
    pub issuer: Option<AddressDb>,
    pub sender: Option<AddressDb>,
    pub created_at: UnixTimestampDb,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl OutputTable for Entity {
    type FilterOptions = AliasFilterOptions;

    fn created_at_col() -> Column {
        Column::CreatedAt
    }

    fn output_id_col() -> Column {
        Column::OutputId
    }
}

impl IndexedOutputTable for Entity {
    fn id_col() -> Column {
        Column::AliasId
    }
}

#[derive(Debug, Default)]
pub(crate) struct AliasFilterOptions {
    state_controller: Option<AddressDb>,
    governor: Option<AddressDb>,
    issuer: Option<AddressDb>,
    sender: Option<AddressDb>,
}

impl Into<sea_query::Cond> for AliasFilterOptions {
    fn into(self) -> sea_query::Cond {
        Cond::all()
            .add_option(self.state_controller.map(|sc| Column::StateController.eq(sc)))
            .add_option(self.governor.map(|governor| Column::Governor.eq(governor)))
            .add_option(self.sender.map(|sender| Column::Sender.eq(sender)))
            .add_option(self.issuer.map(|issuer| Column::Issuer.eq(issuer)))
    }
}

impl TryInto<FilterOptions<Entity>> for AliasFilterOptionsDto {
    type Error = Error;

    fn try_into(self) -> Result<FilterOptions<Entity>, Self::Error> {
        Ok(FilterOptions {
            inner: AliasFilterOptions {
                state_controller: address_dto_option_packed(self.state_controller, "state controller")?,
                governor: address_dto_option_packed(self.governor, "governor")?,
                issuer: address_dto_option_packed(self.issuer, "issuer")?,
                sender: address_dto_option_packed(self.sender, "sender")?,
            },
            pagination: self.pagination.try_into()?,
            timestamp: self.timestamp.try_into()?,
        })
    }
}
