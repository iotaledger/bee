// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use std::{
    marker::PhantomData,
    ops::{Deref, RangeInclusive},
};

macro_rules! make_stages {
    ($($parent_n:ident),+) => {
        $(struct $parent_n;)+
    }
}

macro_rules! impl_multip {
    ($($parent_n:ident),+) => {
        $(impl Parent for $parent_n {})+
        $(impl MultiParent for $parent_n {})+
    }
}

macro_rules! impl_stages {
    ($($parent_n:ident => $parent_m:ident),+) => {
        $(
            impl ParentsBuilder<$parent_n> {
                pub fn add(self, parent: MessageId) -> ParentsBuilder<$parent_m> {
                    ParentsBuilder::<$parent_m>::new(parent, self.parents)
                }
            }
        )+
    };
}

// Needs to be private so it can't be implemented by user code
trait Parent {}
trait MultiParent: Parent {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents(Vec<MessageId>);

impl Parents {
    fn new(message_id: MessageId) -> ParentsBuilder<Parent1> {
        ParentsBuilder::<Parent1>::new(message_id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.0.iter()
    }
}

impl Deref for Parents {
    type Target = Vec<MessageId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<MessageId>> for Parents {
    fn from(mut parents: Vec<MessageId>) -> Self {
        parents.reverse();
        //
        let builder = Parents::new(parents.pop().expect("empty parents vector"));

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        let builder = if let Some(parent) = parents.pop() {
            builder.add(parent)
        } else {
            return builder.finish();
        };

        if let Some(_) = parents.pop() {
            panic!("too many parents");
        } else {
            builder.finish()
        }
    }
}

make_stages!(Parent1, Parent2, Parent3, Parent4, Parent5, Parent6, Parent7, Parent8);
impl Parent for Parent1 {}
impl_multip!(Parent2, Parent3, Parent4, Parent5, Parent6, Parent7, Parent8);
impl_stages!(Parent2 => Parent3, Parent3 => Parent4, Parent4 => Parent5, Parent5 => Parent6, Parent6 => Parent7, Parent7 => Parent8);

impl Packable for Parents {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.len() * MESSAGE_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.len() as u8).pack(writer)?;

        for parent in self.iter() {
            parent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let _parents_len = u8::unpack(reader)? as usize;

        // const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

        // if !MESSAGE_PARENTS_RANGE.contains(&parents_len) {
        //     return Err(Error::InvalidParentsCount(parents_len));
        // }

        // NB: The code doesn't depend on `parents_len`, or its correctness anymore. Instead, the
        // code errors out, if it can not unpack at least 1, or even more than 8 parents. Each
        // `parents` variable is a distinct type according to the builder stage we're in.

        let parents = Parents::new(MessageId::unpack(reader)?);

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        let parents = if let Ok(parent) = MessageId::unpack(reader) {
            parents.add(parent)
        } else {
            return Ok(parents.finish());
        };

        if let Ok(_) = MessageId::unpack(reader) {
            Err(Error::InvalidParentsCount(9))
        } else {
            Ok(parents.finish())
        }
    }
}

struct ParentsBuilder<T: Parent> {
    parents: Vec<MessageId>,
    _phantom: PhantomData<T>,
}

impl<T: Parent> ParentsBuilder<T> {
    pub fn finish(mut self) -> Parents {
        self.parents.shrink_to_fit();

        Parents(self.parents)
    }
}

impl<T: MultiParent> ParentsBuilder<T> {
    fn new(parent: MessageId, mut parents: Vec<MessageId>) -> Self {
        // fail if parent already exists
        if parents.binary_search(&parent).is_ok() {
            panic!("already inserted that parent");
        }

        parents.push(parent);
        parents.sort_unstable();

        Self {
            parents,
            _phantom: PhantomData,
        }
    }
}

impl ParentsBuilder<Parent1> {
    fn new(first: MessageId) -> Self {
        let mut parents = Vec::with_capacity(8);
        parents.push(first);

        Self {
            parents,
            _phantom: PhantomData,
        }
    }

    fn add(self, parent: MessageId) -> ParentsBuilder<Parent2> {
        ParentsBuilder::<Parent2>::new(parent, self.parents)
    }
}
