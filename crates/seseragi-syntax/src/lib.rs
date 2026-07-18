mod cst;
mod declaration;
mod diagnostics;
mod interface;
mod interface_model;
mod lexer;
mod line_continuation;
mod source;
mod surface;
mod surface_model;
mod template;
mod token;

pub use cst::{parse_cst, parse_cst_from_tokens, CstArtifact, CstError, CstMissing, CstNode};
pub use diagnostics::{
    parse_diagnostics, ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticEdit, DiagnosticFix,
    DiagnosticSeverity, RelatedDiagnostic,
};
pub use interface::{
    parse_import_free_module_interface, parse_module_interface, parse_unlinked_module_interface,
    ImportOccurrence, InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport,
    InterfaceInstance, InterfaceMethod, InterfaceOperator, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleHeader, ModuleHeaderName, ModuleInterface, UnlinkedModuleInterface,
};
pub use lexer::lex;
pub use source::{LineColumn, LineIndex, SourceSnapshot, Span};
pub use surface::{
    parse_surface_ast, ByteSpan, SurfaceComprehensionClause, SurfaceConstraint, SurfaceDecl,
    SurfaceDoItem, SurfaceExpr, SurfaceImplMember, SurfaceImport, SurfaceImportItem,
    SurfaceLambdaParameter, SurfaceMatchArm, SurfaceMethod, SurfaceModule, SurfaceParameter,
    SurfacePattern, SurfaceRecordItem, SurfaceRecordPatternField, SurfaceRequirement,
    SurfaceTemplatePart, SurfaceVariant, TypeParameter, TypeRef, Visibility,
};
pub use token::{Token, TokenKind, TokenStream};
