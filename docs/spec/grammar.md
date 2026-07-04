# Appendix A. 文法要約

この EBNF は構文の骨格です。優先順位、型付け、名前解決、網羅性は各章の規則が
優先します。`NEWLINE` は declaration と block item の区切りとして `;` と同等です。

```ebnf
module          = { [ "pub" ], import-decl }, { top-decl } ;

import-decl     = "import", import-items, "from", STRING, terminator
                | "import", "*", "as", lower-name, "from", STRING, terminator ;
import-items    = "{", [ import-item, { ",", import-item } ], "}" ;
import-item     = name, [ "as", name ]
                | "operator", custom-operator,
                  [ "as", custom-operator ] ;

top-decl        = [ "pub" ], let-decl
                | [ "pub" ], fn-decl
                | [ "pub" ], [ "opaque" ], type-decl
                | [ "pub" ], alias-decl
                | [ "pub" ], [ "opaque" ], struct-decl
                | [ "pub" ], trait-decl
                | [ "pub" ], custom-operator-decl
                | impl-decl
                | [ "pub" ], foreign-decl
                | rec-group ;

let-decl        = "let", pattern, [ ":", type ], "=", expr, terminator ;
fn-decl         = "fn", lower-name, [ type-params ], fn-params,
                  "->", type, [ constraints ], fn-body ;
fn-params       = parameter, { "->", parameter } ;
parameter       = lower-name, ":", type-atom ;
fn-body         = "=", expr, terminator | block ;

type-decl       = "type", upper-name, [ type-params ], [ deriving-clause ], "=",
                  variant, { variant }, terminator ;
alias-decl      = "alias", upper-name, [ type-params ], "=", type, terminator ;
variant         = "|", upper-name, [ type ] ;
struct-decl     = "struct", upper-name, [ type-params ], [ deriving-clause ],
                  "{", [ field, { ",", field }, [ "," ] ], "}" ;
field           = lower-name, ":", type ;
deriving-clause = "deriving", upper-name, { ",", upper-name } ;

trait-decl      = "trait", upper-name, type-params, [ constraints ],
                  "{", { trait-method }, "}" ;
trait-method    = "fn", lower-name, [ type-params ], fn-params,
                  "->", type, [ constraints ], terminator ;
impl-decl       = "impl", [ type-params ], type, [ constraints ],
                  "{", { impl-member }, "}" ;
impl-member     = [ "pub" ], fn-decl | overload-decl ;
overload-decl   = "operator", standard-operator, "self",
                  "->", parameter, "->", type, fn-body ;
custom-operator-decl = "operator", [ type-params ], fixity, INTEGER,
                       custom-operator, fn-params, "->", type,
                       [ constraints ], fn-body ;
fixity          = "infixl" | "infixr" | "infix" ;
constraints     = "where", constraint, { ",", constraint } ;
constraint      = upper-name, "<", type-arg, { ",", type-arg }, ">" ;
foreign-decl    = "foreign", STRING, "from", STRING,
                  "{", { foreign-member }, "}" ;
foreign-member  = "opaque", "type", upper-name, [ type-params ], terminator
                | foreign-call
                | "pure", "value", lower-name, ":", foreign-type,
                  [ "=", STRING ], terminator ;
foreign-call    = ( "pure" | "task" ), [ foreign-call-kind ], "fn",
                  lower-name, [ type-params ], foreign-params,
                  "->", foreign-type, [ "=", STRING ], terminator ;
foreign-call-kind = "constructor" | "method" | "property" ;
foreign-params  = parameter, { "->", parameter }, [ "->", rest-parameter ] ;
rest-parameter  = "...", lower-name, ":", type-atom ;
rec-group       = "rec", "{", fn-decl, { fn-decl }, "}" ;

expr            = if-expr | match-expr | do-expr | lambda | assignment-expr ;
if-expr         = "if", expr, "then", expr, "else", expr ;
match-expr      = "match", expr, "{", { match-arm }, "}" ;
match-arm       = pattern, [ "when", expr ], "->", expr, terminator ;
lambda          = "\\", lower-name, [ ":", type ], "->", expr ;
block           = "{", { let-decl }, [ expr ], "}" ;
do-expr         = "do", "{", { do-item, terminator }, expr,
                  [ terminator ], "}" ;
do-item         = pattern, "<-", expr
                | "let", pattern, "=", expr
                | expr ;

primary         = literal | generic-name | operator-reference
                | tuple | array | list | record
                | struct-value
                | block | "(", expr, ")" ;
array           = "[", [ expr, { ",", expr } ], "]"
                | "[", expr, "|", comprehension-clauses, "]" ;
list            = "`[", [ expr, { ",", expr } ], "]"
                | "`[", expr, "|", comprehension-clauses, "]" ;
comprehension-clauses = comprehension-clause,
                        { ",", comprehension-clause } ;
comprehension-clause = pattern, "<-", expr | expr ;
generic-name    = name, [ type-args ] ;
operator-reference = "(", referencable-operator, ")" ;
referencable-operator = arithmetic-operator | comparison-operator | ":"
                     | ">>=" | "<$>" | "<*>" | custom-operator ;
operator        = standard-operator | custom-operator ;
application     = postfix, { postfix } ;
postfix         = primary, { ".", lower-name | "[", expr, "]" } ;
assignment-expr = fallback-expr, [ ":=", assignment-expr ] ;
fallback-expr   = operator-expr, [ "??", fallback-expr ] ;
operator-expr   = unary-expr, { operator, unary-expr } ;
unary-expr      = ( "!" | "-" | "*" ), unary-expr | application ;

pattern         = "_" | literal | lower-name | upper-name, [ pattern ]
                | "(", pattern, ",", pattern, { ",", pattern }, ")"
                | "{", pattern-fields, "}"
                | array-pattern | list-pattern ;
array-pattern   = "[", [ pattern-items ], "]" ;
list-pattern    = "`[", [ pattern-items ], "]" ;
pattern-items   = pattern, { ",", pattern }, [ ",", "...", lower-name ]
                | "...", lower-name ;

type            = function-type ;
function-type   = type-atom, [ "->", function-type ] ;
type-atom       = upper-name, [ "<", type, { ",", type }, ">" ]
                | "(", type, ",", type, { ",", type }, ")"
                | "{", field, { ",", field }, "}"
                | "(", type, ")" ;
type-params     = "<", kind-param, { ",", kind-param }, ">" ;
type-args       = "<", type, { ",", type }, ">" ;
kind-param      = upper-name | upper-name, "<", "_", ">" ;
type-arg        = type | type-constructor ;
type-constructor = upper-name
                 | upper-name, "<", constructor-arg,
                   { ",", constructor-arg }, ">" ;
constructor-arg = type | "_" ;
foreign-type    = type ;

terminator      = NEWLINE | ";" ;
```

## 構文上の確定事項

- 通常の関数適用は空白適用だけです。
- 関数 parameter は `->` でつなぎ、関数型も同じ構文です。
- `if` は必ず `else` を持ちます。
- ADT variant は `|` で始めます。
- pipeline は `|>` です。
- method は `value.method arg` です。
- block は braces で囲み、インデントに意味はありません。
