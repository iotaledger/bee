// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::address::Address;

use crate::{
    dtos::{AddressDto, AliasFilterOptionsInnerDto},
    query::OutputFilterOptions,
    types::{AddressDb, AliasIdDb, AmountDb, UnixTimestampDb},
    Error,
};

use packable::PackableExt;
use sea_orm::{entity::prelude::*, Condition};

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alias_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alias_id: AliasIdDb,
    pub created_at: UnixTimestampDb,
    #[sea_orm(unique)]
    pub output_id: AddressDb,
    pub amount: AmountDb,
    pub state_controller: AddressDb,
    pub governor: AddressDb,
    pub issuer: Option<AddressDb>,
    pub sender: Option<AddressDb>,
}

// The following defintions are need by `sea-orm`.

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Default)]
pub(crate) struct AliasFilterOptions {
    state_controller: Option<AddressDb>,
    governor: Option<AddressDb>,
    issuer: Option<AddressDb>,
    sender: Option<AddressDb>,
}

#[inline(always)]
fn address_dto_option_packed(option: Option<AddressDto>, err_str: &'static str) -> Result<Option<AddressDb>, Error> {
    Ok(option
        .map(|a| Address::try_from(&a).map_err(|_| Error::InvalidField(err_str)))
        .transpose()?
        .map(|a| a.pack_to_vec()))
}

impl TryInto<AliasFilterOptions> for AliasFilterOptionsInnerDto {
    type Error = Error;

    fn try_into(self) -> Result<AliasFilterOptions, Self::Error> {
        Ok(AliasFilterOptions {
            state_controller: address_dto_option_packed(self.state_controller, "state controller")?,
            governor: address_dto_option_packed(self.governor, "governor")?,
            issuer: address_dto_option_packed(self.issuer, "issuer")?,
            sender: address_dto_option_packed(self.sender, "sender")?,
        })
    }
}

impl OutputFilterOptions<Entity, Column> for AliasFilterOptions {
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

        if let Some(state_controller) = self.state_controller {
            filter = filter.add(Column::StateController.eq(state_controller));
        }
        if let Some(governor) = self.governor {
            filter = filter.add(Column::Governor.eq(governor));
        }
        if let Some(sender) = self.sender {
            filter = filter.add(Column::Sender.eq(sender));
        }
        if let Some(issuer) = self.issuer {
            filter = filter.add(Column::Issuer.eq(issuer));
        }

        filter
    }
}
