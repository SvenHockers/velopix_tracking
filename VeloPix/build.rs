// NOTE IF YOUR NOT ON MACOS THEN SORRY THIS WON'T BUILD 
fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-undefined");
        println!("cargo:rustc-link-arg=dynamic_lookup");
    } else if cfg!(target_os = "linux") {
        // For Linux, typically no extra linker flags are needed for a Python extension.
    } else if cfg!(target_os = "windows") {
        // On Windows, you might need specific flags for DLL exports in some cases,
    }
}