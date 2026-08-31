use super::Lockfile;

pub fn write_lockfile(lockfile: &Lockfile) -> String {
    let mut packages = lockfile.packages.iter().collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    let mut output = String::new();
    line(&mut output, "schema", &lockfile.schema.to_string());
    string_line(&mut output, "language", &lockfile.language.to_string());
    string_line(
        &mut output,
        "standard_library",
        &lockfile.standard_library.to_string(),
    );
    string_line(&mut output, "unicode", &lockfile.unicode);
    string_line(
        &mut output,
        "timezone_database",
        &lockfile.timezone_database,
    );
    string_line(&mut output, "root", &lockfile.root);
    for package in packages {
        output.push_str("\n[[packages]]\n");
        string_line(&mut output, "id", &package.id);
        string_line(&mut output, "name", package.name.as_str());
        string_line(&mut output, "version", &package.version.to_string());
        string_line(&mut output, "source_kind", package.source_kind.as_str());
        string_line(&mut output, "source", &package.source);
        string_line(&mut output, "manifest_digest", &package.manifest_digest);
        string_line(&mut output, "content_digest", &package.content_digest);
        let mut dependencies = package.dependencies.iter().collect::<Vec<_>>();
        dependencies.sort_by(|left, right| left.import.as_bytes().cmp(right.import.as_bytes()));
        if dependencies.is_empty() {
            output.push_str("dependencies = []\n");
        } else {
            output.push_str("dependencies = [\n");
            for dependency in dependencies {
                output.push_str("  { import = ");
                output.push_str(&quoted(&dependency.import));
                output.push_str(", package = ");
                output.push_str(&quoted(&dependency.package));
                output.push_str(" },\n");
            }
            output.push_str("]\n");
        }
    }
    let mut foreign_modules = lockfile.foreign_modules.iter().collect::<Vec<_>>();
    foreign_modules.sort_by(|left, right| {
        (
            left.package.as_bytes(),
            left.declaration.as_bytes(),
            left.specifier.as_bytes(),
        )
            .cmp(&(
                right.package.as_bytes(),
                right.declaration.as_bytes(),
                right.specifier.as_bytes(),
            ))
    });
    for foreign in foreign_modules {
        output.push_str("\n[[foreign_modules]]\n");
        string_line(&mut output, "package", &foreign.package);
        string_line(&mut output, "declaration", &foreign.declaration);
        string_line(&mut output, "specifier", &foreign.specifier);
        string_line(&mut output, "exact_identity", &foreign.exact_identity);
        string_line(
            &mut output,
            "declaration_digest",
            &foreign.declaration_digest,
        );
        string_line(&mut output, "content_digest", &foreign.content_digest);
    }
    let mut providers = lockfile.providers.iter().collect::<Vec<_>>();
    providers.sort_by(|left, right| {
        (
            left.service.as_bytes(),
            left.field.as_bytes(),
            left.target.as_bytes(),
            left.provider.as_bytes(),
        )
            .cmp(&(
                right.service.as_bytes(),
                right.field.as_bytes(),
                right.target.as_bytes(),
                right.provider.as_bytes(),
            ))
    });
    for provider in providers {
        output.push_str("\n[[providers]]\n");
        string_line(&mut output, "field", &provider.field);
        string_line(&mut output, "service", &provider.service);
        string_line(
            &mut output,
            "required_contract",
            &provider.required_contract,
        );
        string_line(
            &mut output,
            "provider_contract",
            &provider.provider_contract,
        );
        string_line(&mut output, "provider", &provider.provider);
        string_line(&mut output, "package_version", &provider.package_version);
        string_line(&mut output, "package_source", &provider.package_source);
        string_line(&mut output, "package_digest", &provider.package_digest);
        string_line(&mut output, "artifact_digest", &provider.artifact_digest);
        string_line(&mut output, "backend", &provider.backend);
        line(
            &mut output,
            "backend_abi_major",
            &provider.backend_abi_major.to_string(),
        );
        string_line(&mut output, "target", &provider.target);
        string_line(&mut output, "entry_module", &provider.entry_module);
        string_line(&mut output, "entry_export", &provider.entry_export);
        string_array(&mut output, "runtime_features", &provider.runtime_features);
        let mut hosts = provider.host_packages.iter().collect::<Vec<_>>();
        hosts.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
        if hosts.is_empty() {
            output.push_str("host_packages = []\n");
        } else {
            output.push_str("host_packages = [\n");
            for host in hosts {
                output.push_str("  { name = ");
                output.push_str(&quoted(&host.name));
                output.push_str(", version = ");
                output.push_str(&quoted(&host.version));
                output.push_str(", source = ");
                output.push_str(&quoted(&host.source));
                output.push_str(", content_digest = ");
                output.push_str(&quoted(&host.content_digest));
                output.push_str(" },\n");
            }
            output.push_str("]\n");
        }
    }
    output
}

fn line(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push_str(" = ");
    output.push_str(value);
    output.push('\n');
}

fn string_line(output: &mut String, key: &str, value: &str) {
    line(output, key, &quoted(value));
}

fn string_array(output: &mut String, key: &str, values: &[String]) {
    let mut values = values.to_vec();
    values.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    output.push_str(key);
    output.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&quoted(value));
    }
    output.push_str("]\n");
}

fn quoted(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character => output.push(character),
        }
    }
    output.push('"');
    output
}
