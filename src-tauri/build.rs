// Same XML as tauri-build's default `windows-app-manifest.xml`.
const APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;

fn main() {
    let target = |key| std::env::var(key).unwrap_or_default();
    if target("CARGO_CFG_TARGET_OS") != "windows" || target("CARGO_CFG_TARGET_ENV") != "msvc" {
        tauri_build::build();
        return;
    }

    // tauri-build links its Common-Controls v6 manifest in as a resource for
    // bins only, so `cargo test` executables start without it and die with
    // STATUS_ENTRYPOINT_NOT_FOUND once tauri touches comctl32. Embed the same
    // manifest through the linker instead: a plain rustc-link-arg reaches every
    // linked target (the app, the cdylib and the test executables).
    let manifest = std::path::Path::new(&target("OUT_DIR")).join("app.manifest");
    std::fs::write(&manifest, APP_MANIFEST).expect("write app manifest");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());

    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attrs).expect("tauri-build failed");
}
