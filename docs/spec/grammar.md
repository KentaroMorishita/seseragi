# 文法要約

この EBNF は構文の骨格です。優先順位、型付け、名前解決、網羅性は各章の規則が
優先します。`NEWLINE` は declaration と block item の区切りとして `;` と同等です。

```ebnf
module          = { import-decl }, { top-decl } ;

import-decl     = "import", import-items, "from", STRING, terminator
                | "import", "*", "as", lower-name, "from", STRING, terminator ;
import-items    = "{", [ import-item, { ",", import-item } ], "}" ;
import-item     = name, [ "as", name ] ;

top-decl        = [ "pub" ], let-decl
                | [ "pub" ], fn-decl
                | [ "pub" ], type-decl
                | [ "pub" ], alias-decl
                | [ "pub" ], struct-decl
                | [ "pub" ], trait-decl
                | impl-decl
                | foreign-decl
                | rec-group ;

let-decl        = "let", pattern, [ ":", type ], "=", expr, terminator ;
fn-decl         = "fn", lower-name, [ type-params ], fn-params,
                  "->", type, [ constraints ], fn-body ;
fn-params       = parameter, { "->", parameter } ;
parameter       = lower-name, ":", type-atom ;
fn-body         = "=", expr, terminator | block ;

type-decl       = "type", upper-name, [ type-params ], "=",
                  variant, { variant }, terminator ;
alias-decl      = "alias", upper-name, [ type-params ], "=", type, terminator ;
variant         = "|", upper-name, [ type ] ;
struct-decl     = "struct", upper-name, [ type-params ],
                  "{", [ field, { ",", field }, [ "," ] ], "}" ;
field           = lower-name, ":", type ;

trait-decl      = "trait", upper-name, type-params, [ constraints ],
                  "{", { trait-method }, "}" ;
trait-method    = "fn", lower-name, [ type-params ], fn-params,
                  "->", type, [ constraints ], terminator ;
impl-decl       = "impl", [ type-params ], type, [ constraints ],
                  "{", { fn-decl }, "}" ;
constraints     = "where", constraint, { ",", constraint } ;
constraint      = upper-name, "<", type-arg, { ",", type-arg }, ">" ;
foreign-decl    = "foreign", STRING, "from", STRING,
                  "{", { foreign-fn }, "}" ;
foreign-fn      = [ "pure" ], "fn", lower-name, fn-params,
                  "->", foreign-type, terminator ;
rec-group       = "rec", "{", fn-decl, { fn-decl }, "}" ;

expr            = if-expr | match-expr | lambda | operator-expr ;
if-expr         = "if", expr, "then", expr, "else", expr ;
match-expr      = "match", expr, "{", { match-arm }, "}" ;
match-arm       = pattern, [ "when", expr ], "->", expr, terminator ;
lambda          = "\\", lower-name, [ ":", type ], "->", expr ;
block           = "{", { let-decl }, [ expr ], "}" ;

primary         = literal | name | tuple | array | list | record | struct-value
                | block | "(", expr, ")" ;
application     = postfix, { postfix } ;
postfix         = primary, { ".", lower-name | "[", expr, "]" } ;
operator-expr   = application, { operator, application } ;

pattern         = "_" | literal | lower-name | upper-name, [ pattern ]
                | "(", pattern, ",", pattern, { ",", pattern }, ")"
                | "{", pattern-fields, "}"
                | "[", [ pattern, { ",", pattern } ], "]" ;

type            = function-type ;
function-type   = type-atom, [ "->", function-type ] ;
type-atom       = upper-name, [ "<", type, { ",", type }, ">" ]
                | "(", type, ",", type, { ",", type }, ")"
                | "{", field, { ",", field }, "}"
                | "(", type, ")" ;
type-params     = "<", kind-param, { ",", kind-param }, ">" ;
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
