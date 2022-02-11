// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dtos::{AddressDto, AliasFilterOptionsInnerDto},
    query::OutputFilterOptions,
    types::{AddressDb, AliasIdDb, UnixTimestampDb},
    Error,
};

use sea_orm::{Condition, entity::prelude::*};

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alias_outputs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alias_id: AliasIdDb,
    pub created_at: UnixTimestampDb,
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

#[derive(Debug, Default)]
pub(crate) struct AliasFilterOptions {
    state_controller: Option<AddressDb>,
    governor: Option<AddressDb>,
    issuer: Option<AddressDb>,
    sender: Option<AddressDb>,
}

#[inline(always)]
fn parse_address_option(option: Option<AddressDto>, err_str: &'static str) -> Result<Option<AddressDb>, Error> {
    // TODO: Perform validation
    // option
    //     .map(|sc| sc.parse::<Address>().map_err(|_| Error::InvalidField(err_str)))
    //     .transpose()
    option
        .map(|o| hex::decode(o.0).map_err(|_| Error::InvalidField(err_str)))
        .transpose()
}

impl TryInto<AliasFilterOptions> for AliasFilterOptionsInnerDto {
    type Error = Error;

    fn try_into(self) -> Result<AliasFilterOptions, Self::Error> {
        Ok(AliasFilterOptions {
            state_controller: parse_address_option(self.state_controller, "state_controller")?,
            governor: parse_address_option(self.governor, "governor")?,
            issuer: parse_address_option(self.issuer, "issuer")?,
            sender: parse_address_option(self.sender, "sender")?,
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
