// FILE: build.rs
fn main() {
    let paths: Vec<_> = glob::glob("./services/*.json")
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Tell cargo to re-run this script only when json files in services/
    // have changed. You can choose to omit this step if you want to
    // re-generate services every time.
    for path in &paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    // Generate Rust code and place the output in the src/services/.
    // !! ACHTUNG !! Make sure you use an empty directory or your files may be overwritten.
    krpc_mars_terraformer::run(&paths, "./src/services/").expect("Could not terraform Mars");
}
