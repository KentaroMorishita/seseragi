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
                | [ "pub" ], effect-fn-decl
                | [ "pub" ], [ "opaque" ], type-decl
                | [ "pub" ], [ "opaque" ], newtype-decl
                | [ "pub" ], alias-decl
                | [ "pub" ], [ "opaque" ], struct-decl
                | [ "pub" ], trait-decl
                | [ "pub" ], custom-operator-decl
                | impl-decl
                | instance-decl
                | [ "pub" ], foreign-decl
                | rec-group ;

let-decl        = "let", pattern, [ ":", type ], "=", expr, terminator ;
fn-decl         = "fn", lower-name, [ type-params ], [ fn-params ],
                  "->", type, [ constraints ], fn-body ;
fn-params       = parameter, { "->", parameter } ;
parameter       = lower-name, ":", type-atom ;
fn-body         = "=", expr, terminator | block ;
effect-fn-decl  = "effect", "fn", lower-name, [ type-params ],
                  [ fn-params ], "->", type,
                  [ effect-requirements ], [ effect-failure ],
                  [ constraints ], fn-body ;
effect-requirements = "with", capability, { ",", capability } ;
capability      = upper-name | lower-name, ":", type ;
effect-failure  = "fails", type ;

type-decl       = "type", upper-name, [ type-params ], [ deriving-clause ], "=",
                  variant, { variant }, terminator ;
newtype-decl    = "newtype", upper-name, [ type-params ], [ deriving-clause ],
                  "=", type, terminator ;
alias-decl      = "alias", upper-name, [ type-params ], "=", type, terminator ;
variant         = "|", upper-name, [ type ] ;
struct-decl     = "struct", upper-name, [ type-params ], [ deriving-clause ],
                  "{", [ struct-field, { ",", struct-field }, [ "," ] ], "}" ;
struct-field    = lower-name, ":", type ;
record-type-field = lower-name, [ "?" ], ":", type ;
deriving-clause = "deriving", upper-name, { ",", upper-name } ;

trait-decl      = "trait", upper-name, type-params, [ constraints ],
                  "{", { trait-method }, "}" ;
trait-method    = "fn", lower-name, [ type-params ], [ fn-params ],
                  "->", type, [ constraints ], terminator ;
impl-decl       = "impl", [ type-params ], type, [ constraints ],
                  "{", { impl-member }, "}" ;
impl-member     = [ "pub" ], fn-decl | overload-decl ;
instance-decl   = "instance", [ type-params ], type, [ constraints ],
                  "{", { fn-decl }, "}" ;
overload-decl   = "operator", standard-operator, "self",
                  "->", parameter, "->", type, fn-body ;
custom-operator-decl = "operator", [ type-params ], fixity, INTEGER,
                       custom-operator, fn-params, "->", type,
                       [ constraints ], fn-body ;
fixity          = "infixl" | "infixr" | "infix" ;
constraints     = "where", constraint, { ",", constraint } ;
constraint      = type-name, "<", type-arg, { ",", type-arg }, ">" ;
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
rec-group       = "rec", "{", recursive-fn, { recursive-fn }, "}" ;
recursive-fn    = fn-decl | effect-fn-decl ;

expr            = if-expr | match-expr | do-expr | for-expr
                | lambda | assignment-expr ;
if-expr         = "if", expr, "then", expr, "else", expr ;
match-expr      = "match", expr, "{", { match-arm }, "}" ;
match-arm       = pattern, [ "when", expr ], "->", expr, terminator ;
lambda          = "\\", lambda-param, { lambda-param }, "->", expr ;
lambda-param    = lower-name, [ ":", type-atom ] ;
block           = "{", let-decl, { let-decl }, [ expr ], "}"
                | "{", expr, "}" ;
do-expr         = "do", "{", { do-item, terminator }, expr,
                  [ terminator ], "}" ;
do-item         = pattern, "<-", expr
                | "let", pattern, "=", expr
                | expr ;
for-expr        = "for", pattern, "<-", expr, block ;

primary         = literal | generic-name | operator-reference
                | tuple | array | list | record
                | struct-value
                | block | "(", expr, ")" ;
array           = "[", [ expr, { ",", expr } ], "]"
                | "[", expr, "|", comprehension-clauses, "]" ;
list            = "`[", [ expr, { ",", expr } ], "]"
                | "`[", expr, "|", comprehension-clauses, "]" ;
record          = "{", "}"
                | "{", record-explicit-start,
                  { ",", record-item }, [ "," ], "}"
                | "{", lower-name, ",",
                  [ record-item, { ",", record-item }, [ "," ] ], "}" ;
record-explicit-start = lower-name, ":", expr | "...", expr ;
record-item     = lower-name, [ ":", expr ] | "...", expr ;
struct-value    = constructor-name, [ type-args ], "{",
                  [ struct-item, { ",", struct-item }, [ "," ] ], "}" ;
struct-item     = lower-name, [ ":", expr ] | "...", expr ;
comprehension-clauses = comprehension-clause,
                        { ",", comprehension-clause } ;
comprehension-clause = pattern, "<-", expr | expr ;
generic-name    = name, [ type-args ] ;
operator-reference = "(", referencable-operator, ")" ;
referencable-operator = arithmetic-operator | comparison-operator | ":"
                     | ">>=" | "<$>" | "<*>" | custom-operator ;
application     = postfix, { postfix } ;
postfix         = primary,
                  { ".", name, [ type-args ] | "[", expr, "]" } ;
assignment-expr = low-application-expr, [ ":=", assignment-expr ] ;
low-application-expr = pipeline-expr, [ "$", application-rhs ] ;
application-rhs = if-expr | match-expr | do-expr | lambda
                | low-application-expr ;
pipeline-expr   = fallback-expr,
                  { ( ">>=" | "<$>" | "<*>" | "|>" ), fallback-expr } ;
fallback-expr   = operator-expr, [ "??", fallback-expr ] ;
operator-expr   = unary-expr, { infix-operator, unary-expr } ;
infix-operator  = arithmetic-operator | comparison-operator | ":"
                | "&&" | "||" | custom-operator ;
unary-expr      = ( "!" | "-" | "*" ), unary-expr | application ;

pattern         = "_" | literal | lower-name | struct-pattern
                | constructor-name, [ pattern ]
                | "(", pattern, ",", pattern, { ",", pattern }, ")"
                | record-pattern
                | array-pattern | list-pattern ;
record-pattern  = "{", [ pattern-fields ], "}" ;
struct-pattern  = constructor-name, "{", [ pattern-fields ], "}" ;
pattern-fields  = pattern-field, { ",", pattern-field }, [ "," ] ;
pattern-field   = lower-name, [ "?" ], [ ":", pattern ] ;
array-pattern   = "[", [ pattern-items ], "]" ;
list-pattern    = "`[", [ pattern-items ], "]" ;
pattern-items   = pattern, { ",", pattern }, [ ",", "...", lower-name ]
                | "...", lower-name ;

tuple           = "(", expr, ",", expr, { ",", expr }, ")" ;

type            = function-type ;
function-type   = requirement-merge-type, [ "->", function-type ] ;
requirement-merge-type = type-atom, { "&", type-atom } ;
type-atom       = type-name, [ "<", type, { ",", type }, ">" ]
                | "(", type, ",", type, { ",", type }, ")"
                | "{", [ record-type-field, { ",", record-type-field } ], "}"
                | "(", type, ")" ;
type-params     = "<", kind-param, { ",", kind-param }, ">" ;
type-args       = "<", type, { ",", type }, ">" ;
kind-param      = upper-name | upper-name, "<", "_", ">" ;
type-arg        = type | type-constructor ;
type-constructor = type-name
                 | type-name, "<", constructor-arg,
                   { ",", constructor-arg }, ">" ;
constructor-arg = type | "_" ;
foreign-type    = type ;
type-name       = upper-name
                | lower-name, { ".", lower-name }, ".", upper-name ;
constructor-name = upper-name
                 | lower-name, { ".", lower-name }, ".", upper-name ;

literal         = INTEGER | FLOAT | CHAR | STRING | TEMPLATE_STRING
                | "True" | "False" | "()" ;
name            = lower-name | upper-name ;
lower-name      = LOWER_IDENTIFIER ;
upper-name      = UPPER_IDENTIFIER ;
arithmetic-operator = "+" | "-" | "*" | "/" | "%" | "**" ;
comparison-operator = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
standard-operator = "+" | "-" | "*" | "/" | "%" | "**" | "==" ;
custom-operator = OPERATOR_TOKEN ;

terminator      = NEWLINE | ";" ;
```

`LOWER_IDENTIFIER` と `UPPER_IDENTIFIER` は1.1のUnicode identifier規則を先頭文字の大小で分けた
tokenです。`OPERATOR_TOKEN` は1.8の文字集合・予約token除外規則に従います。INTEGER、FLOAT、CHAR、
STRING、TEMPLATE_STRINGのtoken境界とescapeは1.2に従い、負号は数値tokenではなくunary operatorです。

## 構文上の確定事項

- 通常の関数適用は空白適用だけです。
- 関数 parameter は `->` でつなぎ、関数型も同じ構文です。
- parameterを省略した関数宣言は匿名Unit parameterを一つ持ちます。
- `&`を含むtypeはparse後に2.6のrequirement位置制限を検査します。
- `if` は必ず `else` を持ちます。
- ADT variant は `|` で始めます。
- newtypeは型名と同名の一引数constructorを導入します。
- pipeline は `|>` です。
- method は `value.method arg` です。
- block は braces で囲み、インデントに意味はありません。
- `{}` はempty record、`{ expression }` はblockです。単一field shorthand recordは `{ x, }` と
  trailing commaを付けます。
- record/struct valueはfield shorthandとspreadを持ちます。struct spreadの個数・位置は3.5、
  recordの上書き順は3.7に従います。
- record/struct patternのfieldはshorthandを持ちますが、spreadは持ちません。
