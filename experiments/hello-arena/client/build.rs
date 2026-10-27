//! Stamp the plugin JS of the crates this client links into `web/`, where
//! mq-bundle picks up every `*.js`. Generated, so `web/*.js` is gitignored.

fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("web")?;
    write_if_changed("web/platform.js", miniquad_platform::PLUGIN_JS)?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn write_if_changed(path: &str, body: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() != Some(body) {
        std::fs::write(path, body)?;
    }
    Ok(())
}
