macro_rules! unlock {
    ($mutex:expr) => {{
        $mutex.lock().expect("error taking the lock")
    }};
}

macro_rules! unlock_msg {
    ($mutex:expr, $msg:expr) => {{
        $mutex.lock().expect($msg)
    }};
}

macro_rules! share {
    ($inner:expr) => {{
        std::sync::Arc::new($inner)
    }};
}

macro_rules! share_mut {
    ($inner:expr) => {{
        std::sync::Arc::new(std::sync::Mutex::new($inner))
    }};
}
