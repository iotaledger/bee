# Basic Workflow Document (proposal/draft)

This document outlines the development process of the `bee` project and related
crates.

## Intended Audience

Bee developers and contributors.

## Project Management

New features should enter the project by opening an issue in the github
repository. The issue should outline the intent and functionality, and steps
required to implement it. If the feature is more involved, consider splitting
it up into several distinct pieces, creating one issue for each piece. You can
then create a super issue referencing all individual pieces so that other
contributors can keep track.

The IOTA team will make use of Zenhub, which allows the creation of so-called
epics as super-issues.

**NOTE:** Can we integrate external contributors in zenhub?

## Documentation and comments

All public interfaces should have a descriptive documentation, including an
example that compiles and passes doctests.

All instances of `unsafe` should have a comment as to why its use was
unavoidable.

Try to keep code comments to a minimum. If code needs to be commented to
explain its function it can probably be refactored and be made more simple.

## Language

`bee` is primarily developed in the Rust programming language.

## Formatting

`rustfmt` will be the canonical source of truth for formatting. We use the
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

At the moment, we aim to support `x86_64`
[Rust Tier 1 platforms](https://forge.rust-lang.org/platform-support.html).

Testing will be done against Linux, MacOS, and Windows, and all code should compile
against these three platforms.

## Compiler Settings

We stick with the default `rustc` lints, except for the ones mentioned below.
To not interfer with development, these lints are merely warnings. In PRs these
will be denied. 

```rust
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
```

## Commit Messages

Follow the guidelines laid out in
[Conventional Commits v1.0.0-beta.2](https://www.conventionalcommits.org/en/v1.0.0-beta.2/)

## PR guidelines and workflow

Prior to merging a pull request, the following conditions have to be fulfilled:

+ A pull request should have exactly one concern (one feature, one bug, etc)
    + if a PR attempts to address more than one concern, it should be split
      into two or more PRs 
+ All commits should be signed
+ All code should be well tested
    + what constitutes “well tested” is at the descretion of the author. It is
      suggested to follow TDD when coding. In case of doubt, the review process
      will determine if the implemented tests are sufficient.
+ A PR should be squashed into one commit and have a descriptive message on
  what the code does (see the section on Commit messages)
+ All public interfaces have to be properly tested (see the section on documentation
  and comments)
+ All code should be formatted according to the current `rustfmt` settings
+ Code using `unsafe` will be rejected unless usage is absolutely necessary and
  justified.
+ Building and testing in CI have to pass
+ Only PRs that underwent review by two or more maintainers will be merged

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

## On usage of `unsafe`

Usage of `unsafe` Rust is discouraged. Code that uses it will be rejected
unless it is absolutely necessary and justified by the author.

This also extends to dependencies pulled into the project.

## Continuous Integration (open for discussion)

We use `buildkite` for our CI needs, because it is very flexible, can support
building and testing against all relevant platforms, and because other IOTA
teams are already using it and we have in-house experience.

For now, CI will be run with the following rustc lints set via the `RUSTFLAGS`
environment variable:

+ `RUSTFLAGS="-W rustdoc -W missing-docs -D warnings"`

## Code of conduct

**NOTE:** Which CoC should we adopt? Suggestions include:

+ https://berlincodeofconduct.org/
+ https://opensourcedesign.net/code-of-conduct/
+ https://www.rust-lang.org/policies/code-of-conduct

## License

To be compatible with the guidelines of the Eclipse foundation, all code is to
be licensed under the
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0),
which has to be referenced in every crate of the workspace by copying
[`./LICENSE-APACHE`] to its top level directory. For Rust crates, every
`Cargo.toml` has to contained the line `license = "Apache-2.0"`.
