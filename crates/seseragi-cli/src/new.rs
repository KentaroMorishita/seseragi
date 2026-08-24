use seseragi_project::PackageName;
use std::fs;
use std::path::Path;

const CANONICAL_PACKAGE_NAME: &str = "samples/web-starter";
const MANIFEST: &str = include_str!("../../../examples/samples/web-starter/seseragi.toml");
const APP_SOURCE: &str = include_str!("../../../examples/samples/web-starter/src/app.ssrg");
const MAIN_SOURCE: &str = include_str!("../../../examples/samples/web-starter/src/main.ssrg");

pub(crate) fn new(arguments: &[String]) -> Result<i32, String> {
    let [template, destination] = arguments else {
        return Err("new requires `web` and a destination path".to_owned());
    };
    if template != "web" {
        return Err(format!("unknown project template `{template}`"));
    }
    create_web_project(Path::new(destination))?;
    println!("Created Seseragi Web project at {destination}");
    println!("Next:\n  cd {destination}\n  seseragi dev --open");
    Ok(0)
}

fn create_web_project(destination: &Path) -> Result<(), String> {
    if destination.try_exists().map_err(|error| {
        format!(
            "failed to inspect destination {}: {error}",
            destination.display()
        )
    })? {
        return Err(format!(
            "destination already exists: {}",
            destination.display()
        ));
    }
    let package_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "destination must end in a UTF-8 package name".to_owned())?;
    PackageName::parse(package_name)
        .map_err(|error| format!("invalid package name `{package_name}`: {error}"))?;

    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty());
    if let Some(parent) = parent {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create parent {}: {error}", parent.display()))?;
    }
    fs::create_dir(destination).map_err(|error| {
        format!(
            "failed to create destination {}: {error}",
            destination.display()
        )
    })?;
    if let Err(error) = write_web_project(destination, package_name) {
        let _ = fs::remove_dir_all(destination);
        return Err(error);
    }
    Ok(())
}

fn write_web_project(destination: &Path, package_name: &str) -> Result<(), String> {
    let source = destination.join("src");
    fs::create_dir(&source)
        .map_err(|error| format!("failed to create {}: {error}", source.display()))?;
    let manifest = MANIFEST.replacen(CANONICAL_PACKAGE_NAME, package_name, 1);
    for (path, contents) in [
        (destination.join("seseragi.toml"), manifest.as_str()),
        (source.join("app.ssrg"), APP_SOURCE),
        (source.join("main.ssrg"), MAIN_SOURCE),
    ] {
        fs::write(&path, contents)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    let lockfile = crate::lock::resolved_lockfile(destination)?;
    crate::lock::write(destination, &lockfile)?;
    Ok(())
}
