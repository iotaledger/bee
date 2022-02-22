pub use bee_rest_api::types::dtos::{AddressDto};

use serde::{Deserialize};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationDto {
    pub page_size: Option<u64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampOptionsDto {
    pub created_before: Option<u32>,
    pub created_after: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
pub struct FilterOptionsDto<T> {
    #[serde(flatten)]
    pub inner: T,
    #[serde(flatten)]
    pub timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasFilterOptionsDto {
    pub state_controller: Option<String>,
    pub governor: Option<String>,
    pub issuer: Option<String>,
    pub sender: Option<String>,
    
}

#[derive(Debug, Default, Deserialize)]
pub struct BasicFilterOptionsDto {
    #[serde(flatten)]
    pub timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
pub struct FoundryFilterOptionsDto {
    #[serde(rename(deserialize = "address"))]
    pub unlockable_by_address: Option<String>,
    #[serde(flatten)]
    pub timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
pub struct NftFilterOptionsDto {
    #[serde(flatten)]
    pub timestamp: TimestampOptionsDto,
    #[serde(flatten)]
    pub pagination: PaginationDto,
}
