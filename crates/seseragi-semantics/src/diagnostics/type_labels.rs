pub(super) fn type_label(type_ref: &crate::TypedType) -> String {
    crate::TypeDocument::from_typed_type(type_ref).render(crate::TypeRenderOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_function_and_record_structure_in_diagnostic_labels() {
        let type_ref = crate::TypedType::Function {
            parameter: Box::new(crate::TypedType::Record {
                closed: false,
                fields: vec![crate::TypedRecordField {
                    name: "callback".to_owned(),
                    optional: true,
                    type_ref: crate::TypedType::Function {
                        parameter: Box::new(named("Int")),
                        result: Box::new(named("String")),
                    },
                }],
            }),
            result: Box::new(crate::TypedType::Named {
                name: "Array".to_owned(),
                arguments: vec![named("String")],
            }),
        };

        assert_eq!(
            type_label(&type_ref),
            "{ callback?: (Int -> String), ... } -> Array<String>"
        );
    }

    fn named(name: &str) -> crate::TypedType {
        crate::TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
