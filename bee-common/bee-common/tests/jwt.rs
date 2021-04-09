// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "auth")]
use bee_common::auth::jwt;

#[test]
#[cfg(feature = "auth")]
fn jwt_valid() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        1000,
        b"secret",
    )
    .unwrap();

    assert!(jwt.validate(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        b"secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_to_str_from_str_valid() {
    let jwt = jwt::JsonWebToken::from(
        jwt::JsonWebToken::new(
            String::from("issuer"),
            String::from("subject"),
            String::from("audience"),
            1000,
            b"secret",
        )
        .unwrap()
        .to_string(),
    );

    assert!(jwt.validate(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        b"secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_invalid_issuer() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        1000,
        b"secret",
    )
    .unwrap();

    assert!(!jwt.validate(
        String::from("Issuer"),
        String::from("subject"),
        String::from("audience"),
        b"secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_invalid_subject() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        1000,
        b"secret",
    )
    .unwrap();

    assert!(!jwt.validate(
        String::from("issuer"),
        String::from("Subject"),
        String::from("audience"),
        b"secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_invalid_audience() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        1000,
        b"secret",
    )
    .unwrap();

    assert!(!jwt.validate(
        String::from("issuer"),
        String::from("subject"),
        String::from("Audience"),
        b"secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_invalid_secret() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        1000,
        b"secret",
    )
    .unwrap();

    assert!(!jwt.validate(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        b"Secret"
    ));
}

#[test]
#[cfg(feature = "auth")]
fn jwt_invalid_expired() {
    let jwt = jwt::JsonWebToken::new(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        0,
        b"secret",
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    assert!(!jwt.validate(
        String::from("issuer"),
        String::from("subject"),
        String::from("audience"),
        b"secret"
    ));
}
