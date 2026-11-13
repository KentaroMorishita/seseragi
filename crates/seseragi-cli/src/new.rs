use seseragi_project::PackageName;
use std::fs;
use std::path::Path;

/// A project template. Its files are the same files as a checked-in package,
/// so CI builds and tests exactly what `seseragi new` writes.
struct Template {
    id: &'static str,
    summary: &'static str,
    /// The package name in the checked-in manifest, replaced by the new name.
    canonical_name: &'static str,
    kind: &'static str,
    files: &'static [(&'static str, &'static str)],
    next_steps: &'static [&'static str],
}

const TEMPLATES: &[Template] = &[
    Template {
        id: "cli",
        summary: "a command-line tool for the process and native targets",
        canonical_name: "templates/cli",
        kind: "CLI",
        files: &[
            (
                "seseragi.toml",
                include_str!("../../../examples/templates/cli/seseragi.toml"),
            ),
            (
                ".gitignore",
                include_str!("../../../examples/templates/cli/.gitignore"),
            ),
            (
                "src/main.ssrg",
                include_str!("../../../examples/templates/cli/src/main.ssrg"),
            ),
            (
                "tests/main.ssrg",
                include_str!("../../../examples/templates/cli/tests/main.ssrg"),
            ),
        ],
        next_steps: &[
            "seseragi run . -- Alice",
            "seseragi test",
            "seseragi build . --target native",
        ],
    },
    Template {
        id: "web",
        summary: "a browser app built with Signal",
        canonical_name: "samples/web-starter",
        kind: "Web",
        files: &[
            (
                "seseragi.toml",
                include_str!("../../../examples/samples/web-starter/seseragi.toml"),
            ),
            (
                ".gitignore",
                include_str!("../../../examples/samples/web-starter/.gitignore"),
            ),
            (
                "src/app.ssrg",
                include_str!("../../../examples/samples/web-starter/src/app.ssrg"),
            ),
            (
                "src/main.ssrg",
                include_str!("../../../examples/samples/web-starter/src/main.ssrg"),
            ),
            (
                "tests/app.ssrg",
                include_str!("../../../examples/samples/web-starter/tests/app.ssrg"),
            ),
        ],
        next_steps: &["seseragi dev --open", "seseragi test"],
    },
];

pub(crate) fn template_list() -> String {
    TEMPLATES
        .iter()
        .map(|template| format!("  {:<5} {}\n", template.id, template.summary))
        .collect()
}

pub(crate) fn new(arguments: &[String]) -> Result<i32, String> {
    let [template, destination] = arguments else {
        return Err(format!(
            "new requires a template and a destination path, for example `seseragi new cli my-tool`\n\nTemplates:\n{}",
            template_list().trim_end()
        ));
    };
    let Some(template) = TEMPLATES.iter().find(|candidate| candidate.id == template) else {
        return Err(format!(
            "unknown project template `{template}`\n\nTemplates:\n{}",
            template_list().trim_end()
        ));
    };
    create_project(template, Path::new(destination))?;
    println!(
        "Created Seseragi {} project at {destination}",
        template.kind
    );
    let mut next = format!("Next:\n  cd {destination}");
    for step in template.next_steps {
        next.push_str("\n  ");
        next.push_str(step);
    }
    println!("{next}");
    Ok(0)
}

fn create_project(template: &Template, destination: &Path) -> Result<(), String> {
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
    if let Err(error) = write_project(template, destination, package_name) {
        let _ = fs::remove_dir_all(destination);
        return Err(error);
    }
    Ok(())
}

fn write_project(
    template: &Template,
    destination: &Path,
    package_name: &str,
) -> Result<(), String> {
    for (relative, contents) in template.files {
        let path = destination.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        let contents = if *relative == "seseragi.toml" {
            contents.replacen(template.canonical_name, package_name, 1)
        } else {
            (*contents).to_owned()
        };
        fs::write(&path, contents)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    let lockfile = crate::lock::resolved_lockfile(destination)?;
    crate::lock::write(destination, &lockfile)?;
    Ok(())
}
