# Motivation

As part of the transition to Coordicide it has been decided that the
serialization API used by the Bee ecosystem needs to be redesigned. The main
motivation for this redesign is being able to make most of the crates in Bee
usable in a `no_std` environment.

## The old `Packable` trait

The need for a serialization API existed before Coordicide. Efforts to satisfy
this need culminated with the introduction of the `Packable` trait in the
`bee-common` crate during the Chrysalis part 2 period. Most of the design
decisions behind this trait were done to simplify the serialization of the
[IOTA protocol messages](https://github.com/iotaledger/protocol-rfcs/pull/0017).
The proposed trait was the following

```rust
pub trait Packable {
    type Error: Debug;

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error>;

    fn packed_len(&self) -> usize;

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
```

The main issue with this trait is that it cannot be used in a `no_std`
environment because it depends explicitly on the `std::io` API, whose
transition to the `core` crate has not been decided yet.

Another issue is that the `Error` type is used to represent three different
kinds of errors:

- Serialization: Raised when there are issues while writing bytes.
- Deserialization errors: Raised when there are issues while reading bytes.
- Semantic errors: Raised when the bytes being used to create a value are
  invalid for the data layout of such value.

# Design

## Replacing `std::io`

We introduce two new traits to replace `Read` and `Write` from `std::io`.

First we have the `Unpacker` trait which represents a type that can be used to
read bytes from it.

```rust
pub trait Unpacker: Sized {
    type Error;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}
```

- The `Error` associated type represents any error related to byte
  deserialization.

- The `unpack_bytes` method reads enough bytes from the unpacker to fill
  `bytes` completely. This method must fail if the unpacker does not have
  enough bytes to fulfill the request.

We also have the `Packer` trait which represents a type that can be used to
write bytes into it.

```rust
pub trait Packer {
    type Error;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}
```

- The `Error` associated type represents any error related to byte
  serialization.

- The `pack_bytes` method writes all the bytes from `bytes` into the packer.
  This method must fail if the packer does not have enough space to fulfill the
  request.

Both traits allows us to abstract away any IO operation without relying on
`std::io`. This has the additional benefit of allowing us to pack and unpack
from different kinds of buffers without making specific assumptions about the
implementation of the serialization itself.

## The `Packable` trait

The new `Packable` trait is the following

```rust
pub trait Packable: Sized {
    type Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;

    fn packed_len(&self) -> usize;

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>>;
}
```

- The `Error` associated type represents a semantic error.

- The `pack` method serializes the current value using a `Packer` to write the
  bytes.

- The `packed_len` method returns the length in bytes of the serialized value.
  This has to match the number of bytes written using `pack` to avoid
  inconsistencies.

- The `unpack` method deserializes a value using an `Unpacker` to read the
  bytes.

## Error handling

In addition to the three `Error` associated types mentioned before, we provide
two new helper error types:

- The `UnpackError` type which is an enum wrapping values of either the
  `Packable::Error` or `Unpacker::Error` type.

- The `UnknownTagError` type which can be used if the prefix tag used to
  represent the variant of an enum does not correspond to any variant of such
  enum. As a convention, any `Packable::Error` type for an enum must implement
  `From<UnknownTagError>` (this is not enforced in the trait itself because
  this error is not used for structs).

We also use `core::convert::Infallible` as `Packable::Error` for types whose
unpacking does not have semantic errors, like integers for example. A more
grounded explanation of error handling can be found in the *the `derive` macro*
section.

## `Packable` for basic types

The `Packable` trait is implemented for every integer type by encoding the
value as an array of bytes in little-endian order. Booleans are packed
following Rust's data layout, meaning that `true` is packed as a `1` byte and
`false` as a `0` byte. However, boolean unpacking is less strict and unpacks
any non-zero byte as `true`. Additional implementations of `Packable` are
provided for `Vec<T>`, `Box<[T]>`, `[T; N]` and `Option<T>` if `T` implements
`Packable`.

# Usage

## An example

### Packing

We will implement `Packable` for a type that encapsulates optional integer values (like
`Option<i32>`):

```rust
pub enum Maybe {
    Nothing,
    Just(i32),
}
```
Following the conventions from the [IOTA protocol messages RFC](https://github.com/iotaledger/protocol-rfcs/pull/0017),
we will use an integer prefix as a tag to determine which variant of the enum
is being packed.

```rust
use bee_packable::{
    error::{UnknownTagError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

impl Packable for Maybe {
    type Error = UnknownTagError<u8>;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            Self::Nothing => 0u8.pack(packer),
            Self::Just(value) => {
                1u8.pack(packer)?;
                value.pack(packer)
            },
        }
    }

    fn packed_len(&self) -> usize {
        match self {
            Self::Nothing => 0u8.packed_len(),
            Self::Just(value) => 1u8.packed_len() + value.packed_len(),
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        match u8::unpack(unpacker).map_err(UnpackError::coerce)? {
            0u8 => Ok(Self::Nothing),
            1u8 => Ok(Self::Just(i32::unpack(unpacker).map_err(UnpackError::coerce)?)),
            tag => Err(UnpackError::Packable(UnknownTagError(tag))),
        }
    }
}
```

## The `derive` macro

Another option is to use the derive macro for the `Packable` trait which
provides an implementation equivalent to the one written before.

```rust
use bee_packable::{
    error::{UnknownTagError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

#[derive(Packable)]
#[packable(tag_type = u8)]
pub enum Maybe {
    #[packable(tag = 0)]
    Nothing,
    #[packable(tag = 1)]
    Just(i32),
}
```

This macro introduces a few new concepts:

### Tags for enums

A very common pattern when implementing `Packable` for enums consists in
introducing a prefix value to differentiate each variant of the enumeration
when unpacking, this prefix value is known as `tag`. The type of the `tag` is
specified with the `#[packable(tag_type = ...)]` attribute and it can only be
one of `u8`, `u16`, `u32` or `u64`. The `tag` value used for each variant is
specified with the `#[packable(tag = ...)]` attribute and can only contain
integer literal without any type prefixes (e.g. `42` is valid but `42u8` is
not).

In the example above, the `tag` type is `u8`, the `Nothing` variant has a `tag`
value of `0` and the `Just` variant has a `tag` value of `1`. This means that
the packed version of `Maybe::Nothing` is `[0u8]` and the packed version of
`Maybe::Just(7)` is `[1u8, 0u8, 0u8, 0u8, 7u8]`.

The `tag_type` and `tag` attributes are mandatory for enums. Additionally the
`tag` for each variant must be unique inside the enum.

### Invalid tag values and error handling

Following the example above, unpacking a `Maybe` value that starts with a `tag`
value different from `0` or `1` should fail. To represent this kind of error we
introduced the `UnknownTagError<T>` type. This type is used as
`Packable::Error` when deriving `Packable` for an enum, the `T` type used is
the one specified in the `tag_type` attribute. Additionally, we use the
`std::convert::Infallible` type as `Packable::Error` for structs by default.

However, sometimes it is necessary to use a different error type when deriving
`Packable`. Two examples where this can happen are when `Packable` is being
derived for a type that contains a field which has a custom implementation of
`Packable` or when `Packable` is being derived for a struct whose fields use a
`Packable::Error` type different from `Infallible`. In that case the user can
specify a custom error type using the `#[packable(error = ...)]` attribute. The
type used in this attribute must implement `From<E>` where `E` can be the
`Packable::Error` associated type of any field of the type.

## Optional features

There is only one optional feature for this crate:

- The `io` feature which implements `Packer` for any type that implements
  `std::io::Write` and `Unpacker` for any type that implements `std::io::Read`,
  both implementations use `std::io::Error` as the `Error` associated type.

# Unsolved Issues

## Error coercion

One disadvantage of using `UnpackError` as the error variant returned by the
`Packable::unpack` method is that conversions between errors must be explicit.
As seen in the `Maybe` example, we use the `UnpackError::coerce` method to map
the error variant to an appropiate type. The signature of this method is

```rust
impl<T, U> UnpackError<T, U> {
    pub fn coerce<V: From<T>>(self) -> UnpackError<V, U> { ... }
}
```

and it takes care of mapping the `UnpackError::Packable` variant from `T` to
`V`. At first sight this could be solved by implementing `From<UnpackError<T,
U>>` for `UnpackError<V, U>`. However, that implementation would overlap with
the existing implementations in the standard library.

In a similar fashion we introduced a `UnpackError::infallible` method with the
following signature

```rust
impl<U> UnpackError<Infallible, U> {
    pub fn infallible<V>(self) -> UnpackError<V, U> { ... }
}
```

with the same philosophy. It is not possible to provide a single `coerce`
method for both operations because of the same reasons around using the `From`
trait.

# Alternatives

## Serde

The design of this trait is heavily inspired from the Serde crate and, in fact,
one of the first ideas was to implement a custom `Serializer` and
`Deserializer` for the format used in the IOTA protocol message. However there
were two downsides around this idea:

- Serde does not provide first-class support for arrays of lengths greater than
  `32`. External crates can be used to solve this but it adds an extra
  dependency to the Bee ecosystem.

- Serde is already being used for different types in the Bee ecosystem in
  addition to `Packable`, which means that both serializations are coupled and
  no custom serialization for the IOTA protocol can be provided without
  interfering with the regular serialization to other formats using Serde.

## A `no_std` IO library

At the time there is no de-facto crates for `no_std` IO in Rust. The most
popular one is the [GenIO crate](https://github.com/Kixunil/genio) which is
being redesigned. Thus we cannot count on its stability or maintainaility.
