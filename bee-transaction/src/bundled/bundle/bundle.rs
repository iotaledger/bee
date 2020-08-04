// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    bundled::{Address, BundledTransaction, BundledTransactionField, BundledTransactions},
    Vertex,
};

use bee_crypto::ternary::Hash;

use std::collections::HashMap;

pub struct Bundle(pub(crate) BundledTransactions);

impl Bundle {
    // TODO TEST
    pub fn get(&self, index: usize) -> Option<&BundledTransaction> {
        self.0.get(index)
    }

    // TODO TEST
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // TODO TEST
    pub fn hash(&self) -> &Hash {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap().bundle()
    }

    // TODO TEST
    pub fn tail(&self) -> &BundledTransaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap()
    }

    // TODO TEST
    pub fn head(&self) -> &BundledTransaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(self.len() - 1).unwrap()
    }

    // TODO TEST
    pub fn trunk(&self) -> &Hash {
        self.head().trunk()
    }

    // TODO TEST
    pub fn branch(&self) -> &Hash {
        self.head().branch()
    }

    // TODO TEST
    pub fn ledger_diff(&self) -> HashMap<Address, i64> {
        let mut diff = HashMap::new();

        for transaction in self {
            if *transaction.value.to_inner() != 0 {
                *diff.entry(transaction.address().clone()).or_insert(0) += *transaction.value.to_inner();
            }
        }

        diff
    }
}

impl IntoIterator for Bundle {
    type Item = BundledTransaction;
    type IntoIter = std::vec::IntoIter<BundledTransaction>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (self.0).0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bundle {
    type Item = &'a BundledTransaction;
    type IntoIter = std::slice::Iter<'a, BundledTransaction>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (&(self.0).0).iter()
    }
}

impl std::ops::Index<usize> for Bundle {
    type Output = BundledTransaction;

    // TODO TEST
    fn index(&self, index: usize) -> &Self::Output {
        // Unwrap because index is expected to panic if out of range
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {}
