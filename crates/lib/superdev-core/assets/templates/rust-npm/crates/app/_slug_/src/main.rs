//! The {{superdev:project-name}} binary: parse arguments, call the core
//! library, exit. Keep logic in the core crate, keep this thin.

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") => println!("{{superdev:project-slug}} {}", {{superdev:project-ident}}_core::version()),
        _ => println!("{}", {{superdev:project-ident}}_core::greeting()),
    }
}
