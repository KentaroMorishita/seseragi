use super::*;

pub(super) fn maybe_interface() -> ModuleInterface {
    interface("maybe", "MaybeT", None)
}
pub(super) fn either_interface() -> ModuleInterface {
    interface("either", "EitherT", Some("E"))
}
pub(super) fn reader_interface() -> ModuleInterface {
    interface("reader", "ReaderT", Some("R"))
}
pub(super) fn state_interface() -> ModuleInterface {
    interface("state", "StateT", Some("S"))
}
pub(super) fn writer_interface() -> ModuleInterface {
    interface("writer", "WriterT", Some("W"))
}

fn interface(kind: &str, name: &str, extra: Option<&str>) -> ModuleInterface {
    let module = format!("std/transformer/{kind}");
    let parameters = |with_a: bool| {
        extra
            .into_iter()
            .map(TypeParameter::value)
            .chain([TypeParameter::constructor("M", 1)])
            .chain(with_a.then(|| TypeParameter::value("A")))
            .collect::<Vec<_>>()
    };
    let base = |a| named_with("M", vec![a]);
    let pair = |a, b| InterfaceType::Tuple {
        elements: vec![a, b],
    };
    let transformed = |a| {
        external_type(
            name,
            &format!("{module}::{name}"),
            &module,
            name,
            extra
                .into_iter()
                .map(named)
                .chain([named("M"), a])
                .collect(),
        )
    };
    let representation = match kind {
        "maybe" => base(named_with("Maybe", vec![named("A")])),
        "either" => base(named_with("Either", vec![named("E"), named("A")])),
        "reader" => function_type(vec![named("R")], base(named("A"))),
        "state" => function_type(vec![named("S")], base(pair(named("A"), named("S")))),
        "writer" => base(pair(named("A"), named("W"))),
        _ => unreachable!(),
    };
    let mut ty = type_export(&module, name, parameters(true).len() as u32, "struct");
    ty.scheme.type_parameters = parameters(true);
    ty.representation = Some(record([required("run", representation)]));
    let mut exports = vec![ty];
    let mut add =
        |operation: &str, with_a: bool, constrained: bool, args: Vec<InterfaceType>, result| {
            let mut constraints = vec![];
            if constrained {
                constraints.push(collection_constraint("Monad", vec![named("M")]));
                if kind == "writer" {
                    constraints.push(collection_constraint("Monoid", vec![named("W")]));
                }
            }
            let mut export = function_export(&module, operation, [], constraints, args, result);
            export.scheme.type_parameters = parameters(with_a);
            exports.push(export);
        };
    match kind {
        "maybe" => {
            let inner = named_with("Maybe", vec![named("A")]);
            add(
                "run",
                true,
                false,
                vec![transformed(named("A"))],
                base(inner.clone()),
            );
            add(
                "fromMaybe",
                true,
                true,
                vec![inner],
                transformed(named("A")),
            );
        }
        "either" => {
            let inner = named_with("Either", vec![named("E"), named("A")]);
            add(
                "run",
                true,
                false,
                vec![transformed(named("A"))],
                base(inner.clone()),
            );
            add(
                "fromEither",
                true,
                true,
                vec![inner],
                transformed(named("A")),
            );
        }
        "reader" => {
            add(
                "run",
                true,
                false,
                vec![named("R"), transformed(named("A"))],
                base(named("A")),
            );
            add(
                "ask",
                false,
                true,
                vec![named("Unit")],
                transformed(named("R")),
            );
            add(
                "asks",
                true,
                true,
                vec![function_type(vec![named("R")], named("A"))],
                transformed(named("A")),
            );
            add(
                "local",
                true,
                true,
                vec![
                    function_type(vec![named("R")], named("R")),
                    transformed(named("A")),
                ],
                transformed(named("A")),
            );
        }
        "state" => {
            add(
                "run",
                true,
                false,
                vec![named("S"), transformed(named("A"))],
                base(pair(named("A"), named("S"))),
            );
            add(
                "get",
                false,
                true,
                vec![named("Unit")],
                transformed(named("S")),
            );
            add(
                "put",
                false,
                true,
                vec![named("S")],
                transformed(named("Unit")),
            );
            add(
                "modify",
                false,
                true,
                vec![function_type(vec![named("S")], named("S"))],
                transformed(named("Unit")),
            );
        }
        "writer" => {
            add(
                "run",
                true,
                false,
                vec![transformed(named("A"))],
                base(pair(named("A"), named("W"))),
            );
            add(
                "tell",
                false,
                true,
                vec![named("W")],
                transformed(named("Unit")),
            );
            add(
                "listen",
                true,
                true,
                vec![transformed(named("A"))],
                transformed(pair(named("A"), named("W"))),
            );
        }
        _ => unreachable!(),
    }
    add(
        "lift",
        true,
        true,
        vec![base(named("A"))],
        transformed(named("A")),
    );
    standard_interface(&module, exports)
}
