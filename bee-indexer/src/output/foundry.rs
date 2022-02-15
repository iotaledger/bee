// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::address::Address;
use bee_rest_api::endpoints::routes::api::v1::add_peer;

use crate::{
    dtos::{AddressDto, FoundryFilterOptionsInnerDto},
    query::OutputFilterOptions,
    types::{AddressDb, FoundryIdDb, AmountDb, UnixTimestampDb},
    Error,
};

use packable::PackableExt;
use sea_orm::{entity::prelude::*, Condition};

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "foundry_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub foundry_id: FoundryIdDb,
    pub created_at: UnixTimestampDb,
    #[sea_orm(unique)]
    pub output_id: AddressDb,
    pub amount: AmountDb,
    pub address: AddressDb,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Default)]
pub(crate) struct FoundryFilterOptions {
    unlockable_by_address: Option<AddressDb>,
}

#[inline(always)]
fn address_dto_option_packed(option: Option<AddressDto>, err_str: &'static str) -> Result<Option<AddressDb>, Error> {
    Ok(option
        .map(|a| Address::try_from(&a).map_err(|_| Error::InvalidField(err_str)))
        .transpose()?
        .map(|a| a.pack_to_vec()))
}

impl TryInto<FoundryFilterOptions> for FoundryFilterOptionsInnerDto {
    type Error = Error;

    fn try_into(self) -> Result<FoundryFilterOptions, Self::Error> {
        Ok(FoundryFilterOptions {
            unlockable_by_address: address_dto_option_packed(self.unlockable_by_address, "address")?
        })
    }
}

impl OutputFilterOptions<Entity, Column> for FoundryFilterOptions {
    fn entity() -> Entity {
        Entity
    }

    fn created_at_col() -> Column {
        Column::CreatedAt
    }

    fn output_id_col() -> Column {
        Column::OutputId
    }

    fn filter(self) -> Condition {
        let mut filter = Condition::all();

        if let Some(address) = self.unlockable_by_address {
            filter = filter.add(Column::Address.eq(address));
        }

        filter
    }
}
