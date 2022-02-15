use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct AddressDto(pub(crate) String); // TODO: Use `AddressDto` from bee-rest-api

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
pub struct UniversalFilterOptionsDto {
    #[serde(flatten)]
    pub(crate) timestamp: TimestampOptionsDto, // TODO: is there a TimestampDTO in bee-rest-api?
    #[serde(flatten)]
    pub(crate) pagination: PaginationDto,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasFilterOptionsInnerDto {
    pub(crate) state_controller: Option<AddressDto>,
    pub(crate) governor: Option<AddressDto>,
    pub(crate) issuer: Option<AddressDto>,
    pub(crate) sender: Option<AddressDto>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AliasFilterOptionsDto {
    #[serde(flatten)]
    pub(crate) inner: AliasFilterOptionsInnerDto,
    #[serde(flatten)]
    pub(crate) universal: UniversalFilterOptionsDto,
}

// TODO: Create result DTOs
