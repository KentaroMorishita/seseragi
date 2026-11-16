use super::*;

/// `effects.service`, `effects.provide` and `effects.provideSome`.
pub(super) fn is_environment_operation(name: &str) -> bool {
    matches!(
        name,
        "std/effect::service" | "std/effect::provide" | "std/effect::provideSome"
    )
}

/// Collects the service names of a closed requirement row, as the checker
/// merges them.
fn requirement_fields(ty: &CoreType, fields: &mut BTreeSet<String>) -> Result<()> {
    match ty {
        CoreType::Record {
            closed: true,
            fields: row,
        } => {
            fields.extend(row.iter().map(|field| field.name.clone()));
            Ok(())
        }
        CoreType::RequirementMerge { operands } => operands
            .iter()
            .try_for_each(|operand| requirement_fields(operand, fields)),
        _ => Err("Rust backend: entry environment must be a closed requirement record".into()),
    }
}

impl Backend<'_> {
    /// The environment that `effects.service` reads at the top of the entry
    /// Effect. The TS entry passes `{ console: liveConsole, … }`; native
    /// operations reach their host provider directly, so each service is an
    /// opaque Unit token that user code can only pass on.
    pub(super) fn entry_environment(
        &mut self,
        entry: &seseragi_lowering::CoreFunction,
    ) -> Result<String> {
        let scope = Scope::new();
        let ty = entry
            .value_type
            .as_ref()
            .ok_or("Rust backend: missing checked entry signature")?;
        let CoreType::Function { result, .. } = self.local_canonical_type(ty, &scope)? else {
            return Err("Rust backend: unproven entry function signature".into());
        };
        let CoreType::Named { name, arguments } = *result else {
            return Err("Rust backend: unproven entry Effect type".into());
        };
        if name != "std/prelude::Effect" || arguments.len() != 3 {
            return Err("Rust backend: unproven entry Effect type".into());
        }
        let mut fields = BTreeSet::new();
        requirement_fields(&arguments[0], &mut fields)?;
        let record = self.fresh();
        let mut output = format!("{{ let mut {record} = BTreeMap::new();");
        for field in fields {
            output.push_str(&format!(
                "{record}.insert({}.to_owned(), V::Unit);",
                quoted(&field)
            ));
        }
        output.push_str(&format!("V::Record(Shared::new({record})) }}"));
        Ok(output)
    }
}
