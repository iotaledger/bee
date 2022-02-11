// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dtos::{PaginationDto, UniversalFilterOptionsDto},
    status, Error,
};

use bee_message::{milestone::MilestoneIndex, output::OutputId};

use sea_orm::{ColumnTrait, EntityTrait};

use sea_query::{Cond, Expr, JoinType, Order, SelectStatement};

use std::{mem::size_of, str::FromStr};

pub(crate) type AddressDb = Vec<u8>;
pub(crate) type AliasIdDb = Vec<u8>;
pub(crate) type MilestoneIndexDb = u32;
pub(crate) type UnixTimestampDb = u32;

#[derive(Debug)]
pub(crate) struct CursorPageSize {
    milestone_index: MilestoneIndex,
    output_id: OutputId,
    page_size: Option<u64>,
}

impl FromStr for CursorPageSize {
    type Err = Error;

    fn from_str(cursor: &str) -> Result<Self, Self::Err> {
        // We multiply these values by 2 to get the length in hex.
        let end_milestone_index = 2 * size_of::<MilestoneIndex>();
        let length_output_id = 2 * OutputId::LENGTH;
        let end_output_id = end_milestone_index + length_output_id;

        let milestone_index_str = cursor
            .get(0..end_milestone_index)
            .ok_or(Error::InvalidCursorLength(cursor.len()))?;
        let output_id_str = cursor
            .get(end_milestone_index..end_output_id)
            .ok_or(Error::InvalidCursorLength(cursor.len()))?;

        let milestone_index = u32::from_str_radix(milestone_index_str, 16)
            .map_err(|_| Error::InvalidCursorContent("milestone index"))?
            .into();

        let output_id = output_id_str
            .parse()
            .map_err(|_| Error::InvalidCursorContent("output id"))?;

        // TODO: Clean up
        let page_size = if let Some(".") = cursor.get(end_output_id..end_output_id + 1) {
            let ps: u64 = cursor
                .get((end_output_id + 1)..)
                .ok_or(Error::InvalidCursorLength(cursor.len()))?
                .parse()
                .map_err(|_| Error::InvalidCursorContent("page size"))?;
            Some(ps)
        } else {
            None
        };

        Ok(CursorPageSize {
            milestone_index,
            output_id,
            page_size,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Pagination {
    pub page_size: u64,
    pub cursor: Option<String>,
}

impl TryInto<Pagination> for PaginationDto {
    type Error = Error;

    fn try_into(self) -> Result<Pagination, Self::Error> {
        let mut page_size = self.page_size;
        let mut cursor = None;
        if let Some(cursor_str) = self.cursor {
            // If there is a `page_size` specified in the cursor, it takes precendes over the `page_size` parameter.
            let cursor_page_size = cursor_str.parse::<CursorPageSize>()?;
            cursor = Some(format!(
                "{}{}",
                cursor_page_size.milestone_index, cursor_page_size.output_id
            ));
            if cursor_page_size.page_size.is_some() {
                page_size = cursor_page_size.page_size;
            }
        }
        Ok(Pagination {
            // TODO: We should settle on a sensible default value (from config, maybe)?
            page_size: page_size.unwrap_or(0),
            cursor,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct UniversalFilterOptions {
    // general
    pub created_before: Option<UnixTimestampDb>,
    pub created_after: Option<UnixTimestampDb>,
    pub pagination: Pagination,
}

impl TryInto<UniversalFilterOptions> for UniversalFilterOptionsDto {
    type Error = Error;

    fn try_into(self) -> Result<UniversalFilterOptions, Self::Error> {
        Ok(UniversalFilterOptions {
            created_after: self.timestamp.created_after,
            created_before: self.timestamp.created_before,
            pagination: self.pagination.try_into()?,
        })
    }
}

#[cfg(test)]
mod test {
    use bee_test::rand::{number::rand_number, output::rand_output_id};

    use super::*;

    fn rand_cursor() -> (MilestoneIndex, OutputId, u64, String) {
        let milestone_index = MilestoneIndex(rand_number());
        let milestone_index_enc = hex::encode(milestone_index.to_be_bytes());
        let output_id = rand_output_id();
        let page_size = rand_number();

        let cursor = format!("{milestone_index_enc}{output_id}.{page_size}");
        (milestone_index, output_id, page_size, cursor)
    }

    #[test]
    fn simple_cursor() {
        let milestone_index = MilestoneIndex(rand_number());
        let output_id = rand_output_id();

        let milestone_index_enc = hex::encode(milestone_index.to_be_bytes());

        let cursor_page_size = format!("{milestone_index_enc}{output_id}")
            .parse::<CursorPageSize>()
            .unwrap();

        assert_eq!(cursor_page_size.milestone_index, milestone_index);
        assert_eq!(cursor_page_size.output_id, output_id);
        assert_eq!(cursor_page_size.page_size, None);
    }

    #[test]
    fn simple_cursor_with_page_size() {
        let (milestone_index, output_id, page_size, cursor) = rand_cursor();
        let cursor_page_size = cursor.parse::<CursorPageSize>().unwrap();
        assert_eq!(cursor_page_size.milestone_index, milestone_index);
        assert_eq!(cursor_page_size.output_id, output_id);
        assert_eq!(cursor_page_size.page_size, Some(page_size));
    }

    #[test]
    fn page_size_precedence() {
        let (_, _, page_size, cursor) = rand_cursor();

        let pagination_dto = PaginationDto {
            page_size: Some(42),
            cursor: Some(cursor),
        };

        let result: Pagination = pagination_dto.try_into().unwrap();

        assert_eq!(result.page_size, page_size);
    }
}
