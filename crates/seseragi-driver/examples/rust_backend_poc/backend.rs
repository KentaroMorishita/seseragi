use seseragi_lowering::{
    CoreCallEvidence, CoreDecisionProjection, CoreDecisionTest, CoreExpr, CoreInstanceEvidence,
    CoreInstanceImplementation, CoreModule, CoreParameter, CoreRecordValueItem, CoreStatement,
};
use std::collections::{BTreeMap, BTreeSet};

type Scope = BTreeSet<String>;
type Result<T> = std::result::Result<T, String>;

fn id(name: &str) -> String {
    format!(
        "v_{}",
        name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
fn quoted(value: &str) -> String {
    format!("{value:?}")
}

pub fn emit(module: &CoreModule, entry: &str, iterations: usize) -> Result<String> {
    if iterations == 0 {
        return Err("iterations must be positive".into());
    }
    if !module.foreign_modules.is_empty() || !module.bindings.is_empty() {
        return Err("PoC: foreign modules and top-level bindings are unsupported".into());
    }
    let mut backend = Backend {
        module,
        functions: module.functions.iter().map(|f| f.symbol.clone()).collect(),
        constructors: module
            .adts
            .iter()
            .flat_map(|a| {
                a.variants
                    .iter()
                    .map(|v| (v.symbol.clone(), v.payload.is_some()))
            })
            .collect(),
        serial: 0,
    };
    let entry = module
        .functions
        .iter()
        .find(|f| f.symbol.rsplit("::").next() == Some(entry))
        .ok_or("PoC: entry function not found")?;
    if entry.parameters.len() != 1
        || entry.parameters[0].kind != "implicit"
        || !entry.constraints.is_empty()
    {
        return Err("PoC: entry must be an unconstrained Unit function".into());
    }
    let mut output = String::from("// Generated from checked CoreIr by the opt-in Rust PoC.\n#![allow(dead_code, unused_variables)]\n");
    output.push_str(include_str!("runtime.rs"));
    for (symbol, payload) in &backend.constructors {
        let value = if *payload {
            format!("V::fun(move |payload| V::adt({}, payload))", quoted(symbol))
        } else {
            format!("V::adt({}, V::Unit)", quoted(symbol))
        };
        output.push_str(&format!("\nfn {}() -> V {{ {value} }}\n", id(symbol)));
    }
    for instance in &module.instances {
        if instance.supertrait_count != 0 {
            return Err("PoC: supertraits are unsupported".into());
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("PoC: derived instances are unsupported".into());
        };
        let mut scope = Scope::new();
        let setup = dictionary_parameters(instance.constraints.len(), &mut scope);
        let mut values = Vec::new();
        for method in methods {
            if !method.constraints.is_empty() {
                return Err("PoC: method constraints are unsupported".into());
            }
            let value = backend.function(&method.parameters, &method.body, &scope)?;
            values.push(format!("({}, {value})", quoted(&method.name)));
        }
        output.push_str(&format!(
            "\nfn {}(dicts: Vec<V>) -> V {{ {setup} V::dict(vec![{}]) }}\n",
            id(&instance.identity),
            values.join(",")
        ));
    }
    for function in &module.functions {
        if function.is_effect {
            return Err("PoC: Effect source lowering is unsupported".into());
        }
        let mut scope = Scope::new();
        let setup = dictionary_parameters(function.constraints.len(), &mut scope);
        let body = backend.function(&function.parameters, &function.body, &scope)?;
        output.push_str(&format!(
            "\nfn {}(dicts: Vec<V>) -> V {{ {setup} {body} }}\n",
            id(&function.symbol)
        ));
    }
    output.push_str(&format!(r#"
fn main() {{
    let started = std::time::Instant::now();
    for iteration in 0..{iterations} {{
        {{
            let result = {}(vec![]).apply(V::Unit);
            if iteration == 0 {{ println!("{{}}", result.render()); }}
        }}
        assert_eq!(LIVE.with(Cell::get), 0, "managed values retained after entry return");
    }}
    eprintln!("METRICS iterations={{}} live={{}} peak={{}} allocations={{}} shares={{}} elapsed_us={{}}",
        {iterations}, LIVE.with(Cell::get), PEAK.with(Cell::get), ALLOCATIONS.with(Cell::get),
        SHARES.with(Cell::get), started.elapsed().as_micros());
}}
"#, id(&entry.symbol)));
    Ok(output)
}

fn dictionary_parameters(count: usize, scope: &mut Scope) -> String {
    (0..count)
        .map(|n| {
            let name = format!("\0dictionary_{n}");
            scope.insert(name.clone());
            format!("let {} = dicts[{n}].clone();", id(&name))
        })
        .collect()
}

struct Backend<'a> {
    module: &'a CoreModule,
    functions: BTreeSet<String>,
    constructors: BTreeMap<String, bool>,
    serial: usize,
}

impl Backend<'_> {
    fn fresh(&mut self) -> String {
        self.serial += 1;
        format!("temporary_{}", self.serial)
    }

    fn function(
        &mut self,
        parameters: &[CoreParameter],
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        if let Some((parameter, rest)) = parameters.split_first() {
            let mut inner = scope.clone();
            inner.insert(parameter.id.clone());
            let body = self.function(rest, body, &inner)?;
            Ok(Self::closure(&parameter.id, &body, scope))
        } else {
            self.expr(body, scope)
        }
    }

    fn closure(parameter: &str, body: &str, scope: &Scope) -> String {
        // Owned captures avoid borrowing a caller's stack. Sharing is shallow.
        let clones = scope
            .iter()
            .map(|name| {
                let name = id(name);
                format!("let {name} = {name}.clone();")
            })
            .collect::<String>();
        format!(
            "{{ {clones} V::fun(move |{}| {{ {body} }}) }}",
            id(parameter)
        )
    }

    fn evidence(&mut self, evidence: &[CoreCallEvidence], scope: &Scope) -> Result<String> {
        let mut values = Vec::new();
        for selected in evidence {
            let value = match &selected.evidence {
                CoreInstanceEvidence::Parameter { index } => {
                    let name = format!("\0dictionary_{index}");
                    if !scope.contains(&name) {
                        return Err("PoC: dictionary parameter out of scope".into());
                    }
                    format!("{}.clone()", id(&name))
                }
                CoreInstanceEvidence::Local {
                    identity,
                    evidence_arguments,
                    ..
                } => {
                    if !self
                        .module
                        .instances
                        .iter()
                        .any(|i| &i.identity == identity)
                    {
                        return Err(format!("PoC: unknown local dictionary {identity}"));
                    }
                    let arguments = self.evidence(evidence_arguments, scope)?;
                    format!("{}({arguments})", id(identity))
                }
                other => return Err(format!("PoC: unsupported dictionary {other:?}")),
            };
            values.push(value);
        }
        Ok(format!("vec![{}]", values.join(",")))
    }

    fn reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        if scope.contains(name) {
            if !evidence.is_empty() {
                return Err("PoC: constrained local callable aliases are unsupported".into());
            }
            Ok(format!("{}.clone()", id(name)))
        } else if self.functions.contains(name) {
            let args = self.evidence(evidence, scope)?;
            Ok(format!("{}({args})", id(name)))
        } else if self.constructors.contains_key(name) {
            if !evidence.is_empty() {
                return Err("PoC: constructor evidence unsupported".into());
            }
            Ok(format!("{}()", id(name)))
        } else {
            Err(format!("PoC: unsupported reference {name}"))
        }
    }

    fn expr(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        Ok(match expr {
            CoreExpr::Unit { .. } => "V::Unit".into(),
            CoreExpr::Integer { value, .. } => {
                let value = value.parse::<i64>().map_err(|_| "PoC: invalid integer")?;
                format!("checked_int({value}_i64)")
            }
            CoreExpr::String { value, .. } => format!("V::text({})", quoted(value)),
            CoreExpr::Boolean { value, .. } => format!("V::Bool({value})"),
            CoreExpr::Variable { name, evidence, .. } => self.reference(name, evidence, scope)?,
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => {
                if !deferred_evidence_parameters.is_empty()
                    || !deferred_evidence_type_constructor_parameters.is_empty()
                    || evidence_argument_index.is_some()
                {
                    return Err("PoC: deferred callable evidence is unsupported".into());
                }
                let function = if let Some(dispatch) = trait_dispatch {
                    if evidence.len() != 1 {
                        return Err("PoC: trait dispatch requires one dictionary".into());
                    }
                    let dictionary = self.evidence(evidence, scope)?;
                    format!(
                        "({dictionary}).remove(0).field({})",
                        quoted(&dispatch.method)
                    )
                } else {
                    self.reference(callee, evidence, scope)?
                };
                let args = self.expressions(arguments, scope)?;
                format!("apply({function}, vec![{args}])")
            }
            CoreExpr::Lambda {
                parameter, body, ..
            } => self.function(std::slice::from_ref(parameter), body, scope)?,
            CoreExpr::Tuple { elements, .. } => format!(
                "V::Tuple(Shared::new(vec![{}]))",
                self.expressions(elements, scope)?
            ),
            CoreExpr::Array { elements, .. } => format!(
                "V::Array(Shared::new(vec![{}]))",
                self.expressions(elements, scope)?
            ),
            CoreExpr::FieldAccess {
                receiver, field, ..
            } => format!("({}).field({})", self.expr(receiver, scope)?, quoted(field)),
            CoreExpr::Record { items, .. } => {
                let mut values = Vec::new();
                for item in items {
                    match item {
                        CoreRecordValueItem::Field { name, value, .. } => values.push(format!(
                            "({}.to_owned(), {})",
                            quoted(name),
                            self.expr(value, scope)?
                        )),
                        _ => return Err("PoC: record spread is unsupported".into()),
                    }
                }
                format!(
                    "V::Record(Shared::new(BTreeMap::from([{}])))",
                    values.join(",")
                )
            }
            CoreExpr::Binary {
                operator,
                left,
                right,
                evidence,
                ..
            } => {
                let standard = evidence.iter().all(|e| matches!(&e.evidence, CoreInstanceEvidence::Standard { identity, .. } if matches!(identity.as_str(), "std/int::Add" | "std/int::Sub" | "std/int::Mul" | "std/int::Eq" | "std/int::Ord" | "std/string::Add")));
                if !standard || !matches!(operator.as_str(), "+" | "-" | "*" | "==" | "<" | "<=") {
                    return Err(format!(
                        "PoC: unsupported operator/evidence {operator} {evidence:?}"
                    ));
                }
                format!(
                    "binary({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                )
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => format!(
                "if ({}).boolean() {{ {} }} else {{ {} }}",
                self.expr(condition, scope)?,
                self.expr(then_branch, scope)?,
                self.expr(else_branch, scope)?
            ),
            CoreExpr::Decision {
                scrutinee,
                branches,
                ..
            } => {
                let temporary = self.fresh();
                let label = self.fresh();
                let mut value = format!(
                    "{{ let {temporary} = {}; '{label}: {{",
                    self.expr(scrutinee, scope)?
                );
                for branch in branches {
                    let conditions = branch
                        .tests
                        .iter()
                        .map(|test| self.test(test, &temporary))
                        .collect::<Result<Vec<_>>>()?;
                    value.push_str(&format!(
                        "if {} {{",
                        if conditions.is_empty() {
                            "true".into()
                        } else {
                            conditions.join(" && ")
                        }
                    ));
                    let mut inner = scope.clone();
                    for binding in &branch.bindings {
                        value.push_str(&format!(
                            "let {} = {};",
                            id(&binding.name),
                            Self::project(&temporary, &binding.path)?
                        ));
                        inner.insert(binding.name.clone());
                    }
                    let guard = branch
                        .guard
                        .as_ref()
                        .map(|g| self.expr(g, &inner))
                        .transpose()?
                        .map(|g| format!("({g}).boolean()"))
                        .unwrap_or_else(|| "true".into());
                    value.push_str(&format!(
                        "if {guard} {{ break '{label} {}; }} }}",
                        self.expr(&branch.value, &inner)?
                    ));
                }
                value.push_str("panic!(\"PoC: unmatched checked decision\"); } }");
                value
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => {
                let mut value = String::from("{");
                let mut inner = scope.clone();
                for statement in statements {
                    match statement {
                        CoreStatement::PureLet {
                            name,
                            value: expression,
                            ..
                        } => {
                            value.push_str(&format!(
                                "let {} = {};",
                                id(name),
                                self.expr(expression, &inner)?
                            ));
                            inner.insert(name.clone());
                        }
                        _ => return Err("PoC: only pure let statements are supported".into()),
                    }
                }
                value.push_str(&self.expr(result, &inner)?);
                value.push('}');
                value
            }
            _ => {
                return Err(format!(
                    "PoC: unsupported CoreIr expression at {:?}",
                    std::mem::discriminant(expr)
                ))
            }
        })
    }

    fn expressions(&mut self, values: &[CoreExpr], scope: &Scope) -> Result<String> {
        Ok(values
            .iter()
            .map(|v| self.expr(v, scope))
            .collect::<Result<Vec<_>>>()?
            .join(","))
    }

    fn project(root: &str, path: &[CoreDecisionProjection]) -> Result<String> {
        let mut value = format!("{root}.clone()");
        for projection in path {
            value = match projection {
                CoreDecisionProjection::AdtPayload => format!("({value}).payload()"),
                CoreDecisionProjection::TupleElement { index }
                | CoreDecisionProjection::ArrayElement { index } => {
                    format!("({value}).index({index})")
                }
                CoreDecisionProjection::RecordField { name } => {
                    format!("({value}).field({})", quoted(name))
                }
                _ => return Err("PoC: unsupported decision projection".into()),
            };
        }
        Ok(value)
    }

    fn test(&self, test: &CoreDecisionTest, root: &str) -> Result<String> {
        Ok(match test {
            CoreDecisionTest::Constructor {
                path, constructor, ..
            } => format!(
                "({}).tagged({})",
                Self::project(root, path)?,
                quoted(constructor)
            ),
            CoreDecisionTest::Integer { path, value, .. } => format!(
                "matches!({}, V::Int(n) if n == {}_i64)",
                Self::project(root, path)?,
                value
                    .parse::<i64>()
                    .map_err(|_| "PoC: invalid decision integer")?
            ),
            CoreDecisionTest::Boolean { path, value, .. } => format!(
                "matches!({}, V::Bool(n) if n == {value})",
                Self::project(root, path)?
            ),
            _ => return Err("PoC: unsupported decision test".into()),
        })
    }
}
