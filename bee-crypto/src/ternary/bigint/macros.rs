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

macro_rules! def_and_impl_ternary {
    ($ident:ident, $len:expr) => {
        /// Length of the trit buffer.
        pub const LENGTH: usize = $len;

        /// Fixed size trit buffer.
        #[derive(Clone, Debug)]
        pub struct $ident<T: Trit>(TritBuf<T1B1Buf<T>>);

        impl<T: Trit> $ident<T> {
            /// Creates a new fixed size trit buffer from input.
            pub fn new(trits_buf: TritBuf<T1B1Buf<T>>) -> Self {
                assert_eq!(trits_buf.len(), LENGTH);
                $ident(trits_buf)
            }

            /// Creates a new trit buffer that represents the value 0.
            pub fn zero() -> Self {
                Self(TritBuf::zeros(LENGTH))
            }

            /// Transforms into its inner trit buffer.
            pub fn into_inner(self) -> TritBuf<T1B1Buf<T>> {
                self.0
            }
        }

        impl<T: Trit> std::ops::Deref for $ident<T> {
            type Target = TritBuf<T1B1Buf<T>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: Trit> std::ops::DerefMut for $ident<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<T> $ident<T>
        where
            T: Trit,
            <T as ShiftTernary>::Target: Trit,
        {
            /// Transforms into a shifted representation of the trit buffer.
            pub fn into_shifted(self) -> $ident<<T as ShiftTernary>::Target> {
                $ident(self.0.into_shifted())
            }
        }

        impl $ident<Btrit> {
            /// Creates a new balanced trit buffer that represents the value 1.
            pub fn one() -> Self {
                let mut trits = Self::zero();
                trits.0.set(0, Btrit::PlusOne);
                trits
            }

            /// Creates a new balanced trit buffer that represents the value -1.
            pub fn neg_one() -> Self {
                let mut trits = Self::zero();
                trits.0.set(0, Btrit::NegOne);
                trits
            }

            /// Creates a new balanced trit buffer that represents the maximum value.
            pub fn max() -> Self {
                Self(TritBuf::filled(LENGTH, Btrit::PlusOne))
            }

            /// Creates a new balanced trit buffer that represents the minimum value.
            pub fn min() -> Self {
                Self(TritBuf::filled(LENGTH, Btrit::NegOne))
            }
        }

        impl $ident<Utrit> {
            /// Creates a new unbalanced trit buffer that represents the value 1.
            pub fn one() -> Self {
                let mut trits = Self::zero();
                trits.0.set(0, Utrit::One);
                trits
            }

            /// Creates a new unbalanced trit buffer that represents the value 2.
            pub fn two() -> Self {
                let mut trits = Self::zero();
                trits.0.set(0, Utrit::Two);
                trits
            }

            /// Creates a new unbalanced trit buffer that represents the half of the maximum value.
            pub fn half_max() -> Self {
                Self(TritBuf::filled(LENGTH, Utrit::One))
            }

            /// Creates a new unbalanced trit buffer that represents the maximum value.
            pub fn max() -> Self {
                Self(TritBuf::filled(LENGTH, Utrit::Two))
            }

            /// Creates a new unbalanced trit buffer that represents the minimum value.
            pub fn min() -> Self {
                Self::zero()
            }
        }

        impl<T: Trit> Default for $ident<T> {
            fn default() -> Self {
                Self::zero()
            }
        }

        impl<T: Trit> Eq for $ident<T> {}

        impl<T: Trit> PartialEq for $ident<T> {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl<T: Trit> Ord for $ident<T> {
            fn cmp(&self, other: &Self) -> Ordering {
                match self.partial_cmp(other) {
                    Some(ordering) => ordering,
                    // Cannot be reached because the order is total.
                    None => unreachable!(),
                }
            }
        }

        impl<T: Trit> PartialOrd for $ident<T> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                use Ordering::Equal;
                for (a, b) in self.0.iter().zip(other.0.iter()).rev() {
                    match a.cmp(&b) {
                        Equal => continue,
                        other_ordering => return Some(other_ordering),
                    }
                }
                Some(Equal)
            }
        }
    };
}

macro_rules! impl_const_functions {
    ( ( $($root:tt)* ), { $endianness:ty $(,)? }, { $repr:ty $(,)? } ) => {
        impl $($root)* < $endianness, $repr > {
            /// Creates an instance from an array of inner representation.
            pub const fn from_array(inner: $repr) -> Self {
                Self {
                    inner,
                    _phantom: PhantomData,
                }
            }
        }
    };

    ( ( $($root:tt)* ), { $endianness:ty $(,)? }, { $repr:ty, $( $rest:ty ),+ $(,)? } ) => {

        impl_const_functions!( ( $($root)* ), { $endianness }, { $repr } );
        impl_const_functions!( ( $($root)* ), { $endianness }, { $( $rest ),+ } );
    };

    ( ( $($root:tt)* ), { $endianness:ty, $( $rest:ty ),+ $(,)? }, { $( $repr:ty ),+ $(,)? } ) => {

        impl_const_functions!( ( $($root)* ), { $endianness }, { $( $repr ),+ });

        impl_const_functions!( ( $($root)* ), { $( $rest ),+ }, { $( $repr ),+ });
    };
}

macro_rules! impl_constants {
    ( $( $t:ty => [ $( ( $fn:ident, $val:expr ) ),+ $(,)? ]),+ $(,)? ) => {
        $(
            impl $t {
                $(
                    /// Returns the appropriate constant value.
                    pub const fn $fn() -> Self {
                        $val
                    }
                )+
            }
        )+
    };
}

macro_rules! impl_toggle_endianness {
    ( @inner
      ( $($root:tt)* ),
      $repr:ty,
      $src_endian:ty,
      $dst_endian:ty
    ) => {
        impl From< $($root)* < $src_endian, $repr >> for $($root)* <$dst_endian, $repr> {
            fn from(value: $($root)*<$src_endian, $repr>) -> Self {
                let mut inner = value.inner;
                inner.reverse();
                Self {
                    inner,
                    _phantom: PhantomData,
                }
            }
        }
    };

    ( ( $($root:tt)* ), $head:ty $(,)?) => {
        impl_toggle_endianness!(@inner ($($root)*), $head, LittleEndian, BigEndian);
        impl_toggle_endianness!(@inner ($($root)*), $head, BigEndian, LittleEndian);
    };

    ( ( $($root:tt)* ), $head:ty, $( $tail:ty ),+ $(,)?) => {
        impl_toggle_endianness!( ( $($root)* ), $head );
        impl_toggle_endianness!( ( $($root)* ), $( $tail ),+ );
    };
}
