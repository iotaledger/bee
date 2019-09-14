pub(crate) const BROADCAST_BUFFER_SIZE: usize = 2;

pub(crate) mod messages {
    pub(crate) const SYSTEM_NOT_INITIALIZED_ERROR: &str =
        "fatal error: the system doesn't seem to be properly initialized";

    pub(crate) const ENVIRONMENT_ALREADY_EXISTS_ERROR: &str =
        "error: an environment with that name already exists";

    pub(crate) const ENVIRONMENT_DOESNT_EXIST_ERROR: &str =
        "error: an environment with that name does not exist";

    pub(crate) const ENTITY_DOESNT_EXIST_ERROR: &str =
        "error: an entity with that id does not exist";

    pub(crate) const UNLOCK_SUPERVISOR_ERROR: &str = "error: the supervisor could not be unlocked";

    pub(crate) const UNLOCK_SHUTDOWN_HANDLER_ERROR: &str =
        "error: the shutdown handler could not be unlocked";

    pub(crate) const UNLOCK_RUNTIME_ERROR: &str = "error: the runtime could not be unlocked";

    pub(crate) const UNLOCK_TERMINATION_CHAN_ERROR: &str =
        "error: the termination channel could not be unlocked";

    pub(crate) const SUPERVISOR_RUNTIME_ERROR: &str =
        "runtime error: caused by the supervisor future";

    pub(crate) const ENVIRONMENT_RUNTIME_ERROR: &str =
        "runtime error: caused by an environment future";

    pub(crate) const ENTITY_RUNTIME_ERROR: &str = "runtime error: caused by an entity future";

    pub(crate) const ENTITY_ALREADY_JOINED_ERROR: &str =
        "This entity already joined that environment";

    pub(crate) const ENTITY_ALREADY_AFFECTS_ERROR: &str =
        "This entity already affects that environment";

    pub(crate) const RUNTIME_SHUTDOWN_ERROR: &str =
        "error: the runtime could not be shut down properly";

    pub(crate) const RUNTIME_START_ERROR: &str = "error: the runtime could not be started properly";

    pub(crate) const SEND_TERMINATION_SIGNAL_ERROR: &str =
        "error: termination signal could not be sent.";

    pub(crate) const EMIT_SIGNAL_ERROR: &str = "error: a signal could not be broadcasted";
}
