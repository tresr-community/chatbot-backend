// This macro is only enabled for debug builds.
#[macro_export]
macro_rules! console_trace {
    ($( $args:expr ),*) => {
        #[cfg(debug_assertions)]
        {
            console_debug!( $( $args ),* );
        }
    };
}
