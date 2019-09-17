# Contributing to bee

This document outlines the development process of the `bee` project and its
constituent crates. It explains how its core development team operates, and
how community members can contribute.

## Outline

+ Language
+ Features, issues, pull requests
+ Documentation and comments
+ Formatting
+ Versioning
+ Testing
+ Toolchains and suppported platforms
+ Compiler Settings
+ Commit Messages
+ PR guidelines and workflow
+ On `unsafe`
+ Continuous Integration (open for discussion)
+ License

## Language

At this time, the `bee` framework is implemented in the Rust programming language.
All software documentation, and comments, and all communication between contributors
is to be held in English.

## Features, issues, pull requests

If you want to propose a new feature, we ask you to write an RFC (Request for
Comments) document and submit it in the form of a pull request to the
[`iotaledger/bee-rfcs`](https://github.com/iotaledger/bee-rfcs) repository. The
readme of that repository outlines all necessary steps.

If there is a new feature in the form of an accepted RFC waiting to be
implemented, please comment on its tracking issue to indicate your interest to
avoid double work.

If you find a bug in one of the crates, please open a Github issue on the
[`iotaledger/bee`](https://github.com/iotaledger/bee) repository. Please also
open an issue prior to submitting a bugfix.

Minor cosmetic changes like fixing a typo or the wording of a comment or piece
of documentation can but do not need to have an issue opened prior to providing
a fix.

New features should enter the project by opening an issue in the github
repository. The issue should outline the intent and functionality, and steps
required to implement it. If the feature is more involved, consider splitting
it up into several distinct pieces, creating one issue for each piece. You can
then create a super issue referencing all individual pieces so that other
contributors can keep track.

## Documentation and comments

All public interfaces should have a descriptive documentation, including an
example that compiles and passes doctests.

All instances of `unsafe` should have a comment as to why its use was
unavoidable.

Try to keep code comments to a minimum. If code needs to be commented to
explain its function it can probably be refactored and be made more simple.

## Formatting

`rustfmt` is the canonical source of truth for formatting. We use the
default formatting options as defined by `rustfmt`, except for `edition
= "2018"`.

If you want to prevent rustfmt from formatting code you have organized by hand
(say macros, attributes, or lookup tables), you can do so by using the
attribute `#[rustfmt::skip]`. A more finegrained configuration is possible, see
the [rustfmt tips](https://github.com/rust-lang/rustfmt#tips).

## Versioning

We follow [semantic versioning 2.0](https://semver.org/), as is standard in
Rust. For a short description on how it's used in Rust, see
[this section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field)
in the cargo manifest.

## Testing

All code should be well tested, using unit tests and integration tests. All
public interfaces should contain a doctest both to document usage, and to
verify that the example code actually works.

The review process of a pull request will determine if tests are sufficient, or
if more tests should be added.

We encourage test driven development to specify the intended input/output of
a piece of code. 

## Toolchains and suppported platforms

At the moment, all `x86_64`
[Rust Tier 1 platforms](https://forge.rust-lang.org/platform-support.html) are supported.
Code that doesn't compile or pass testing on either of these platforms will not be merged.

Testing will be done against Linux, MacOS, and Windows.

## Compiler Settings

We stick with the default `rustc` lints, except for the ones mentioned below.
To not interfer with development, these lints are merely warnings. In PRs these
will be denied using the setting `RUSTFLAGS="-D warnings"`.

```rust
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
```


## Commit Messages

Commit messages should clearly outline what has been implemented or changed. If
a commit addresses an issue opened in the repository, it should contain one of
the closing keywords as outlined
[here](https://help.github.com/en/articles/closing-issues-using-keywords), for
example `close`, `fix`, or `resolve`.

## PR guidelines and workflow

Prior to merging a pull request, the following conditions have to be fulfilled:

+ A pull request should have exactly one concern (one feature, one bug, etc)
    + if a PR attempts to address more than one concern, it should be split
      into two or more PRs 
    + A new feature can only be implemented if it has an accompanying accepted
      RFC and an associated tracking issue, which the PR has to reference.
    + A bug fix can only be merged if there is an accompanying issue raising it.i
+ All code should be well tested
    + what constitutes “well tested” is at the descretion of the author. It is
      suggested to follow TDD when coding. In case of doubt, the review process
      will determine if the implemented tests are sufficient.
+ A PR should be squashed into one commit and have a descriptive message on
  what the code does (see the section on Commit messages)
+ All code should be formatted according to the current `rustfmt` settings
+ Code using `unsafe` will be rejected unless usage is necessary and justified.
+ CI Builds and tests have to pass.
+ Only PRs that underwent review by at least one core maintainer will be merged.

The typical life cycle of a new pull request looks like this:

1. Fork the repository
2. Create a new branch in your fork
3. Commit changes and push to your fork
4. Create a pull request against `upstream/master`
5. Code review takes place (potentially suggesting more changes, commits)
7. Rebase on recent master/format/squash commits
8. If no build fails and all tests pass, the PR gets approved and merged into
   master

For now, we will merge PRs manually. Once the project grows in size and it
becomes necessary, we will introduce a mergebot such as `bors-ng`.

## On `unsafe`

Usage of `unsafe` Rust is discouraged. Code that uses it will be rejected
unless it is absolutely necessary and justified by the author.

This also extends to dependencies pulled into the project.

## License

To be compatible with the guidelines of the Eclipse foundation, all code is to
be licensed under the
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0),
which has to be referenced in every crate of the workspace by copying
[`./LICENSE-APACHE`] to its top level directory. For Rust crates, every
`Cargo.toml` has to contained the line `license = "Apache-2.0"`.
