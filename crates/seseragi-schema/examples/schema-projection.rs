//! Actual second-source conversion. The only DTS API used is the public common
//! graph projector; no TypeScript source or declaration parser is invoked.
fn main() {
    let source = include_str!("../tests/fixtures/customer.schema.json");
    let settings = "schema = 1\n[entries.api]\ndeclaration = 'customer.schema.json'\noutput = 'customer'\nspecifier = './customer.js'\nevaluation = 'pure'\n";
    let graph =
        seseragi_schema::interpret_schema(source, "customer.schema.json", "Customer", settings)
            .unwrap();
    let projected = seseragi_dts::project_graph(&graph, settings, "api").unwrap();
    assert!(
        projected.diagnostics.is_empty(),
        "{:?}",
        projected.diagnostics
    );
    println!("{}", projected.source.unwrap());
}
