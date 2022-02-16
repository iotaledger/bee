use crate::{types::AddressDb, Error};

pub(crate) mod alias;
pub(crate) mod basic;
pub(crate) mod foundry;
pub(crate) mod nft;

#[inline(always)]
pub(crate) fn address_dto_option_packed(option: Option<String>, err_str: &'static str) -> Result<Option<AddressDb>, Error> {
    Ok(option
        .map(|a| hex::decode(&a).map_err(|_| Error::InvalidField(err_str)))
        .transpose()?)
}
