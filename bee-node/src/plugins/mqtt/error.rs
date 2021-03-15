use librumqttd as mqtt;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Mqtt operation failed: {0}.")]
    Mqtt(#[from] mqtt::Error),
}
