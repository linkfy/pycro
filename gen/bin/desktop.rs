//! Desktop launcher that delegates to the shared pycro runtime entrypoint.

fn main() {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pycro::main();
}
