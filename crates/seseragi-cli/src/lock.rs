use std::fs;
use std::path::Path;

pub(crate) fn lock(arguments: &[String]) -> Result<i32, String> {
    let path = match arguments {
        [command] if command == "update" => Path::new("."),
        [command, path] if command == "update" => Path::new(path),
        _ => return Err("lock requires `update` and an optional package path".to_owned()),
    };
    let lockfile = resolved_lockfile(path)?;
    write(path, &lockfile)?;
    println!("Updated {}", path.join("seseragi.lock").display());
    Ok(0)
}

pub(crate) fn resolved_lockfile(path: &Path) -> Result<seseragi_project::Lockfile, String> {
    let mut lockfile = seseragi_project::generate_lockfile(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    for target in seseragi_project::ProjectTarget::ALL {
        match crate::local_project::compile_path_unlocked(
            path,
            seseragi_project::ProjectCommand::Build,
            Some(target),
        ) {
            Ok(crate::local_project::LocalProjectCompilation::Compiled(compiled)) => {
                lockfile.providers.extend(
                    compiled
                        .compiled
                        .compiled
                        .provider_resolution
                        .as_ref()
                        .map(|resolution| resolution.lock.project_lock_selections())
                        .unwrap_or_default(),
                );
            }
            Ok(crate::local_project::LocalProjectCompilation::Diagnostics) => break,
            Err(_) => {}
        }
    }
    Ok(lockfile)
}

pub(crate) fn write(path: &Path, lockfile: &seseragi_project::Lockfile) -> Result<(), String> {
    let output = seseragi_project::write_lockfile(&lockfile);
    let destination = path.join("seseragi.lock");
    fs::write(&destination, output)
        .map_err(|error| format!("failed to write {}: {error}", destination.display()))?;
    Ok(())
}
