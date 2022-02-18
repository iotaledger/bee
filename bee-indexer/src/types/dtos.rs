pub use bee_rest_api::types::dtos::{AddressDto};

use serde::{Deserialize};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationDto {
    pub(crate) page_size: Option<u64>,
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampOptionsDto {
    pub(crate) created_before: Option<u32>,
    pub(crate) created_after: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasFilterOptionsDto {
    pub(crate) state_controller: Option<String>,
    pub(crate) governor: Option<String>,
    pub(crate) issuer: Option<String>,
    pub(crate) sender: Option<String>,
    #[serde(flatten)]
    pub(crate) timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub(crate) pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
pub struct BasicFilterOptionsDto {
    #[serde(flatten)]
    pub(crate) timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub(crate) pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
pub struct FoundryFilterOptionsDto {
    #[serde(rename(deserialize = "address"))]
    pub(crate) unlockable_by_address: Option<String>,
    #[serde(flatten)]
    pub(crate) timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub(crate) pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
pub struct NftFilterOptionsDto {
    #[serde(flatten)]
    pub(crate) timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub(crate) pagination: PaginationDto,
}
