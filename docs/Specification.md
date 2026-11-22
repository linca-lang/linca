---

## title: Linca Core Language Specification version: 0.1 status: draft date: 2026-11-20 tags: [linca, spec]

# Linca Core Language Specification

**Version:** 0.1 (draft) **Status:** for discussion **Scope:** the language only. The IR, plugin protocol and CLI are specified separately.
---

## 1. Overview

Linca is a declarative, non-Turing-complete language for describing data models and service interfaces once and generating consistent artifacts (ORM models, database schemas, API contracts) for multiple languages through plugins.

Design constraints:

1. **No computation.** No functions, conditionals, loops or macros. Everything a user writes expands to plain declarations before the IR is built. Every valid file can be fully analyzed statically.
2. **Composition, not inheritance.** Field reuse is done with `mixin`; polymorphism with `union`.
3. **Target-neutral core.** The core knows nothing about specific languages or ORMs. Target-specific details live in namespaced attributes (`@prisma.map`, `@sqlalchemy.table`). How a construct is represented in a target is defined by the plugin protocol specification, not by this document. The one transport the core knows is HTTP: the binding of operations to HTTP is part of the language, because every plugin has to agree on it ([[#17. Services]]).
4. **Errors are part of the language.** Every rule in this document that can be violated has a diagnostic code (Section 12).

---

## 2. Source files

- Extension: `.linca`.
- Encoding: UTF-8. A BOM MAY be present and is ignored.
- Line endings: LF or CRLF.
- Each file is a module (Section 10).

---

## 3. Lexical structure

### 3.1 Comments

```
// line comment
/* block comment, does not nest */
/// doc comment, attached to the next declaration, field or operation
```

A doc comment is equivalent to a `@doc("...")` attribute on the item that follows it. Several consecutive `///` lines are joined with newlines. A doc comment not followed by a declaration, field or operation is an error (E0105).

### 3.2 Identifiers

```
ident = (letter | "_") { letter | digit | "_" } ;
letter = "a".."z" | "A".."Z" ;
digit  = "0".."9" ;
```

Identifiers are ASCII only. Naming conventions are checked as warnings, not errors:

|Item|Convention|Example|
|---|---|---|
|Types (`model`, `record`, `mixin`, `enum`, `union`, `view`, `type`)|PascalCase|`OrderItem`|
|Fields|camelCase|`createdAt`|
|Services, errors|PascalCase|`Users`, `NotFound`|
|Operations, parameters|camelCase|`getUser`|
|Enum variants|PascalCase|`Pending`|
|Attributes|camelCase|`sensitive`|
|Modules|snake_case|`shop::order_items`|

Generators convert names to each target's conventions; the source convention is fixed.

### 3.3 Keywords

Keywords:

```
model record mixin enum union view type attribute repeatable import
with without only partial requires as true false null on
service error throws
```

Reserved for future versions:

```
override include extend for private
```

Keywords are **contextual**. A keyword has its special meaning only in the positions where the grammar expects it.

- A keyword (current or reserved) MAY be used as a field name, a parameter name (of an operation or of an attribute declaration) and an attribute argument name without any escaping: `type: String`, `on: Bool` (ADR-0002). The only exceptions are `true`, `false` and `null`, which are always literals.
- A keyword MUST NOT be used as the name of a declaration, an enum variant, an operation or a module segment (E0104).

This is unambiguous because a field always starts with `field_name ":"` inside a block, a parameter with `field_name ":"` inside the parentheses of an operation or an attribute declaration, and declarations always start at file level. There is no escaping syntax.

### 3.4 Literals

```
string  = '"' { char | escape } '"' ;
escape  = "\\" ( '"' | "\\" | "n" | "t" | "r" | "u{" hex { hex } "}" ) ;
int     = [ "-" ] digit { digit | "_" } ;
float   = [ "-" ] digit { digit } "." digit { digit } [ ("e"|"E") [ "+"|"-" ] digit { digit } ] ;
bool    = "true" | "false" ;
null    = "null" ;
list    = "[" [ literal { "," literal } [ "," ] ] "]" ;
```

A string MUST NOT contain a raw line break (LF or CR LF); write the escape sequence backslash-n instead. A line break before the closing quote is E0102 with the primary span starting at the opening quote; the string ends there and lexing resumes on the next line (ADR-0003).

How a literal is checked against a type is defined in [[#7.7 Argument values]].

### 3.5 Punctuation

```
{ } ( ) [ ] < > : , = | ? @ . :: + ->
```

---

## 4. Grammar

EBNF. Whitespace and comments are allowed between any two tokens. Newlines are ordinary whitespace and are not significant.

Fields, enum variants, operations and declarations have **no separators**. The grammar does not need them: a field always starts with `field_name ":"` and an attribute with `@`, so the parser always knows where the previous item ends. An operation starts with `@` or with `name "("`. A comma or `;` between fields is an error (E0106). Commas are used only inside lists: attribute arguments, `[...]` literals, attribute sets, `{ a, b }` in views and imports, operation parameters and `throws` lists. `linca fmt` puts one field per line.

```
file          = { import } { declaration } ;

import        = "import" path [ "::" "{" ident { "," ident } [ "," ] "}" ] ;
path          = ident { "::" ident } ;

declaration   = { attribute_use } decl_body ;
decl_body     = model | record | mixin | enum | union | view | newtype
              | service | error
              | attribute_decl | attribute_set ;

model         = "model"  ident [ with_clause ] "{" { field } "}" ;
record        = "record" ident [ with_clause ] "{" { field } "}" ;
mixin         = "mixin"  ident [ with_clause ] "{" { field } "}" ;
with_clause   = "with" type_name { "," type_name } ;

field         = field_name ":" type_ref { attribute_use } ;
field_name    = ident | keyword ;                (* any keyword except true, false, null; see 3.3 *)

enum          = "enum" ident "{" { variant } "}" ;
variant       = ident [ "=" ( string | int ) ] { attribute_use } ;

union         = "union" ident [ requires_clause ] "=" member { "|" member } ;
requires_clause = "requires" type_name { "," type_name } ;
member        = type_name [ "as" string ] ;

view          = "view" ident "=" type_name { view_op } ;
view_op       = "without" "{" name_list "}"
              | "only" "{" name_list "}"
              | "+" "{" { field } "}"
              | "partial" ;
name_list     = field_name { "," field_name } [ "," ] ;

newtype       = "type" ident "=" type_ref ;       (* base MUST be a built-in scalar; see 5.5, ADR-0001 *)

service       = "service" ident "{" { operation } "}" ;
operation     = { attribute_use } ident "(" [ op_param { "," op_param } [ "," ] ] ")"
                [ "->" type_ref ] [ "throws" type_name { "," type_name } ] ;
op_param      = field_name ":" type_ref { attribute_use } ;

error         = "error" ident [ with_clause ] "{" { field } "}" ;

attribute_decl = [ "repeatable" ] "attribute" attr_path [ "(" param { "," param } [ "," ] ")" ] "on" target_list ;
attribute_set  = "attribute" ident "on" target_list "=" "[" attribute_use { "," attribute_use } [ "," ] "]" ;
param         = field_name ":" type_ref [ "=" value ] ;   (* ADR-0002 *)
target_list   = target { "," target } ;
target        = "field" | "model" | "record" | "mixin" | "enum" | "variant" | "union" | "view" | "type"
              | "service" | "operation" | "param" | "error" ;

type_ref      = base_type [ "?" ] ;
base_type     = type_name | "List" "<" type_ref ">" | "Map" "<" "String" "," type_ref ">" ;
type_name     = path ;

attribute_use = "@" attr_path [ "(" [ arg { "," arg } [ "," ] ] ")" ] ;
attr_path     = ident { "." ident } ;
arg           = [ field_name ":" ] value ;
value         = literal | path | generator | value_list ;
value_list    = "[" [ value { "," value } [ "," ] ] "]" ;
generator     = ident "(" ")" ;
```

Notes:

- Declaration-level attributes are written **before** the declaration. This holds for every declaration, including `type`. Field-level and variant-level attributes are written **after** the type or variant.
- Operation attributes are written **before** the operation, like declaration attributes. Parameter attributes are written **after** the parameter type, like field attributes. An operation has no attributes after it, so an `@` that follows a result type or a `throws` list always starts the next operation.
- An attribute set lists its attributes in brackets: `attribute id on field = [@primary, @default(uuid())]`. The brackets make the end of the set unambiguous, because the next declaration may also start with `@`.
- Attribute names use `.` for namespaces (`@prisma.map`); module paths use `::`.
- An `import` path without braces always names a module. A single item is imported with braces: `import shop::orders::{Order}`.
- How a `value` is resolved (literal, enum variant, field name) depends on the parameter it is bound to; see [[#7.7 Argument values]].
- `generator` is a built-in value generator, not a function call (see [[#7.6 Value generators]]). Users cannot declare functions or generators.
- An attribute declaration (`attribute_decl`) may have parameters and no `=`. An attribute set (`attribute_set`) has `=` and no parameters, so a set with parameters is a syntax error.

---

## 5. Type system

### 5.1 Built-in scalar types

|Type|Meaning|
|---|---|
|`String`|UTF-8 text|
|`Int`|32-bit signed integer|
|`Int64`|64-bit signed integer|
|`Float`|64-bit IEEE 754|
|`Decimal`|fixed-point decimal; precision set via `@decimal(p, s)`|
|`Bool`|boolean|
|`Uuid`|RFC 9562 UUID|
|`DateTime`|instant with time zone (UTC on the wire)|
|`Date`|calendar date without time|
|`Bytes`|binary data|
|`Json`|arbitrary JSON value (escape hatch)|

A `Decimal` field MUST have `@decimal(p, s)`, either on the field or on its `type` declaration (E0222). There is no default precision, because targets disagree on one.

### 5.2 Collections

- `List<T>` — ordered sequence.
- `Map<String, T>` — string-keyed map. Keys other than `String` are not allowed (E0204).

### 5.3 Optionality

A field has two independent properties:

- **Nullability.** `T?` means the value may be null (in storage and on the wire). Set only by `?`.
- **Presence.** A field is either _required_ (it must be provided) or _optional_ (it may be omitted). Every field written in a `model`, `record` or `mixin` is required. Only the `partial` view operator makes fields optional ([[#9.2 Views]]).

The two are never merged. `name: String?` after `partial` is "may be omitted; if provided, may be null", so an update payload can distinguish "do not change" from "set to null". The IR records both properties for every field.

**A nullable collection is **not allowed**:

|Written|Meaning|Allowed|
|---|---|---|
|`tags: List<String>`|always a list, possibly empty: `[]`, `["a", "b"]`|yes|
|`tags: List<String?>`|always a list; single elements may be null: `["a", null, "b"]`|yes|
|`tags: List<String>?`|three states: no list (`null`), empty list, list with items|**no** (E0205)|

Reason: the difference between "no list" and "empty list" is almost never needed, and targets handle it inconsistently (some ORMs do not support nullable lists, JSON distinguishes `null` from `[]`, typed languages need extra null checks). An empty collection therefore means "nothing". The same rule applies to `Map`.

### 5.4 Nominal typing

Types are nominal: two records with identical fields are different types. A `view` creates a new type, not an alias (Section 9). The same holds for `type` declarations ([[#5.5 Domain types]]).

The compiler compares types in two different ways:

- **Type identity is nominal.** Used wherever two types must be the same: a foreign key and the referenced key in `@relation`, union members against a required mixin. A mismatch is E0220.
- **Values are checked against the underlying scalar.** `@default(...)`, `@length`, `@pattern`, `@format`, `@min`, `@max` on a field of type `UserId` are checked as if the field were `Uuid`.

### 5.5 Domain types

`type` declares a **new nominal type over a built-in scalar** (a "newtype"), optionally with constraints. Attributes are written before the declaration, as for every other declaration:

```
type UserId = Uuid
type OrderId = Uuid

@decimal(12, 2) @min(0)
type Money = Decimal
```

Rules:

- The base MUST be a built-in scalar (Section 5.1), not nullable. A collection (`type Tags = List<String>`), a nullable type (`type Name = String?`) and a type over a record, enum or another `type` are E0221 (ADR-0001). The parser accepts any `type_ref` here; the check reports the error.
- `UserId` and `OrderId` are different types even though both are `Uuid` (see 5.4). `authorId: Uuid` referencing `id: UserId` is E0220.
- Attributes on the declaration apply to every field of that type. A field MUST NOT repeat a non-repeatable attribute its type already carries (E0404).

A domain type stays a distinct type in the IR. Which targets can represent it as a distinct type, and how, is defined by the plugin protocol specification.

**`type` or attribute set?** Use `type` when the value needs its own type in application code (identifiers, money). Use an attribute set ([[#7.4 User-defined attributes]]) when only shared rules are needed and the field should stay a plain scalar:

```
attribute email on field = [@format(Email), @length(max: 320)]

model User {
  email: String @email
}
```

`type` always creates a new type in the IR; an attribute never does.

### 5.6 Where types may be used

|Type kind|As field type in `model`|As field type in `record`|In `List`/`Map`|In an operation signature|
|---|---|---|---|---|
|Scalar, enum, `type`|yes|yes|yes|yes|
|`record`|yes, needs storage strategy|yes|yes|yes|
|`union`|yes, needs storage strategy|yes|yes|yes|
|`model`|yes — becomes a relation (Section 8)|no (E0210)|only `List<Model>` in a model, as a back-relation|no (E0806)|
|`mixin`|no (E0211)|no|no|no (E0211)|
|`view`|no in models (E0212)|yes|yes|yes|
|`error`|no (E0803)|no (E0803)|no (E0803)|only after `throws`|

The fields of an `error` follow the `record` column. "In an operation signature" means a parameter type or a result type ([[#17. Services]]).

---

## 6. Declarations

### 6.1 `model`

A stored entity with identity.

```
@index(fields: [email, createdAt])
model User with Timestamps {
  id: Uuid @primary @default(uuid())
  email: String @unique
  name: String?
  role: Role @default(Member)
}
```

Rules:

- A model MUST have exactly one primary key: one field with `@primary`, or a model-level `@primary(fields: [...])` (E0301, E0302).
- Field names MUST be unique within a declaration, including fields received from mixins (E0304, E0603). The same rule applies to `record` and `mixin`.
- A field with a `record`, `union` or collection type gets the default storage strategy `json`. Another strategy is set with `@storage(...)` (Section 7.3).

### 6.2 `record`

A structured value without identity and without its own storage (DTOs, embedded values, API payloads).

```
record Address {
  street: String
  city: String
  postalCode: String?
}
```

Records MUST NOT have `@primary`, `@unique`, `@index`, `@relation`, `@storage` or `@updated` (E0303). This also applies to fields a record receives from a mixin; the error is reported at the `with` clause.

### 6.3 `mixin`

A named group of fields reused by `model`, `record` or another [[#9.1 Mixins | mixin ]]. A mixin is not a type and cannot be used as a field type.

```
mixin Timestamps {
  createdAt: DateTime @default(now())
  updatedAt: DateTime @updated
}
```

### 6.4 `enum`

```
enum Role {
  Admin
  Member
  Guest = "guest"
}
```

- Variants without a value are stored by name, i.e. as strings.
- Explicit values MUST all be strings or all be ints within one enum (E0310).
- String enums MAY mix variants with and without values (`Guest = "guest"` next to `Admin`). Int enums MUST give every variant a value (E0312).
- Duplicate names or values are errors (E0311).
- The variant's value (explicit, or its name) is what is stored and what goes on the wire.

### 6.5 `union`

A tagged union of records.

```
union Payment requires PaymentBase = CardPayment as "card" | BankTransfer
```

- Members MUST be `record` types (E0320). **[Open]** unions of models (table-per-type storage).
- The discriminator field is named `kind` by default. Another name is set with `@discriminator("method")` on the union.
- The discriminator value of a member defaults to the member's name. `as "card"` sets another value for this union only, so the same record MAY be a member of several unions with different values.
- Discriminator values MUST be unique within a union (E0322).
- The discriminator name MUST NOT be the name of a field of any member (E0323).
- `requires Mixin` requires every member to contain the mixin's fields; the core checks this (E0321). It lets generators produce a shared interface or base type. `requires` does not copy fields, unlike `with` on a model, record or mixin.

### 6.6 `view`

See [[#9.2 Views]].

### 6.7 `type`

See [[#5.5 Domain types]].

### 6.8 `attribute`

See [[#7.4 User-defined attributes]].

### 6.9 `service`

See [[#17.2 Operations]].

### 6.10 `error`

See [[#17.4 Errors]].

---

## 7. Attributes

### 7.1 General rules

- Attributes are metadata. The core validates them; generators interpret them.
- Every attribute used MUST be declared: built in (7.2), by the user (7.4) or by a plugin manifest (7.5). Unknown attributes are errors (E0401).
- An attribute has a set of allowed targets; using it elsewhere is E0402.
- Arguments are bound and type-checked against the declaration (E0403); see [[#7.7 Argument values]].
- An attribute MAY appear at most once per item unless its declaration is `repeatable`. In the core only `@index`, model-level `@unique(fields: [...])` and `@rule` are repeatable. User attributes opt in: `repeatable attribute tag(name: String) on field`.
- Duplicates are counted after attribute sets are expanded: with `attribute id on field = [@primary, @default(uuid())]`, writing `id: Uuid @id @primary` repeats `@primary` and is E0404.

Attributes fall into four levels. The level decides who must act on an attribute:

|Level|Examples|Who acts on it|
|---|---|---|
|Core|`@primary`, `@length`, `@sensitive`|the core checks it; every plugin of the listed backend kinds MUST honor it (7.2)|
|Capability-gated|`@storage(table)`, `@default(autoincrement())`|the core checks, before generation, that every configured target declares the capability ([[#16. Capability-gated constructs]])|
|Plugin|`@prisma.map`|only the plugin that owns the namespace (7.5)|
|User|`@pii`|passed to all plugins in the IR; a plugin that does not know it ignores it (7.4)|

### 7.2 Core attributes

"Honored by" lists the backend kinds (Storage, Types, API) whose plugins MUST reflect the attribute in their output. The exact obligations are defined by the plugin protocol specification.

|Attribute|Targets|Meaning|Honored by|
|---|---|---|---|
|`@primary`|field, model (`fields:`)|primary key|Storage|
|`@unique`|field, model (`fields:`)|uniqueness constraint|Storage|
|`@index(fields: [...])`|model|non-unique index (repeatable)|Storage|
|`@default(value)`|field, param|default value: literal, enum variant or value generator (7.6); on a parameter only a literal or enum variant|Storage, Types, API for literals and enum variants; Storage for value generators|
|`@updated`|field (`DateTime`)|set on every update|Storage|
|`@relation(...)`|field|relation details (Section 8)|Storage|
|`@storage(json \| columns \| table)`|field|storage strategy for record, union and collection fields|Storage|
|`@discriminator(name)`|union|discriminator field name|Storage, Types, API|
|`@sensitive`|field|must not appear in views or API outputs unless explicitly allowed|the core (E0611); passed to all|
|`@allowSensitive`|view, operation|the view may contain `@sensitive` fields; the operation may return them (17.8)|the core|
|`@doc(text)`|any|documentation|Types, API|
|`@min`, `@max`, `@length`, `@pattern`, `@format`|field, `type`, param|validation constraints, see signatures below|Types, API; Storage for `@length` on `String`|
|`@decimal(precision, scale)`|field, `type` (`Decimal`)|precision|Storage, Types, API|
|`@rule(name)`|model, record, view|named validation rule implemented in application code (repeatable)|Types|
|`@route(path)`|service|path prefix of the operations of the service (17.5)|API|
|`@get`, `@post`, `@put`, `@patch`, `@delete`|operation|HTTP method and path of the operation (17.5)|API|
|`@query`, `@header(name)`, `@body`|param|where the parameter is sent, when the default placement does not fit (17.5)|API|
|`@status(code)`|operation, error|HTTP status code (17.5)|API|
|`@readonly`|operation|the operation changes no state (17.7)|API|

`@primary` and `@unique` have two forms: without arguments on a field, and with `fields: List<FieldRef>` on a model.

Reserved for future core use: `@deprecated`, `@since`, `@example`.

Signatures of the validation attributes:

|Attribute|Signature|Applies to|
|---|---|---|
|`@min`|`@min(value)`|numeric scalars; inclusive|
|`@max`|`@max(value)`|numeric scalars; inclusive|
|`@length`|`@length(exact: Int?, min: Int?, max: Int?)`; `@length(4)` binds `exact`|`String`, `Bytes`, `List`|
|`@pattern`|`@pattern(regex: String)`|`String`|
|`@format`|`@format(format: Format)`|`String`|
|`@rule`|`@rule(name: String)`|model, record, view|

`@length` MUST have at least one argument, and `exact` MUST NOT be combined with `min` or `max` (E0403).

`@pattern` always matches the **whole** value, so anchors (`^`, `$`) are not needed. Patterns SHOULD use the common subset of regular expressions (character classes, groups, alternation, quantifiers) so every target engine accepts them; features such as lookbehind or backreferences are a warning (W0405).

`@format` names a well-known string format from the closed built-in enum `Format`: `Email`, `Uri`, `Hostname`, `Ipv4`, `Ipv6`. Users and plugins cannot add formats.

`@rule` declares a validation rule that cannot be expressed declaratively (cross-field or conditional rules). The core does not interpret it. The name MUST be a valid identifier (E0403) and unique within the declaration (E0409). Generators emit a hook with this name that application code implements. Rules are not carried from a source into a view, because the view may lack the fields the rule needs; a view declares its own rules.

### 7.3 Storage strategies

|Strategy|Meaning|Allowed for|
|---|---|---|
|`json` (default)|one JSON column|record, union, collections|
|`columns`|fields flattened into prefixed columns|record (not collections)|
|`table`|separate table with foreign key|record, union, collections|

The strategies are the variants of the built-in enum `StorageStrategy`. The core checks that every target declares the strategy as a capability; otherwise E0420 points at the field in the `.linca` file.

### 7.4 User-defined attributes

```
enum PiiLevel { Low High }

/// Personal data that needs protection.
attribute pii(level: PiiLevel = High) on field
```

User attributes are passed to all generators in the IR. A generator that does not know an attribute ignores it.

A user attribute name MUST NOT contain a namespace (E0407); namespaces belong to plugins (7.5).

**Attribute sets** give a name to a combination of attributes:

```
attribute id on field = [@primary, @default(uuid())]

model Order {
  id: Uuid @id
}
```

A set expands to its attributes before the IR is built; the IR records the set name in `origin`. Rules:

- Attribute sets have no parameters in v0.1; the grammar does not allow them ([[#4. Grammar]]). A set cannot be `repeatable`.
- Every attribute in a set MUST allow every target of the set (E0402).
- A set MAY contain other sets. Cycles are errors (E0410).

### 7.5 Plugin attributes

A plugin declares its attributes in its manifest as ordinary attribute declarations with a namespaced name:

```
attribute prisma.map(name: String) on field, model
```

Rules:

- The namespace equals the plugin name. The core has no namespaced attributes of its own.
- A namespaced attribute MUST NOT be declared in a user module (E0407).
- Using a namespaced attribute of a plugin that is not configured is a warning (W0401), not an error, so a schema can be shared between projects with different targets. Its arguments are not checked.

Keeping plugin attributes in separate overlay files (`extend model User for prisma { ... }`) is planned for v0.2; `extend` and `for` are reserved for it.

### 7.6 Value generators

`uuid()`, `uuid7()`, `now()` and `autoincrement()` look like calls but are **not functions**: the compiler executes nothing. They are built-in markers meaning "this value is produced when the record is inserted". In the IR they become data such as `{ "generated": "uuid" }`, and each plugin maps them to its target.

The list is closed; users and plugins cannot add generators. Generators take no arguments.

|Generator|Allowed field types (or a `type` over them)|Meaning|
|---|---|---|
|`uuid()`|`Uuid`|random UUID (version 4)|
|`uuid7()`|`Uuid`|time-ordered UUID (version 7)|
|`now()`|`DateTime`, `Date`|current time|
|`autoincrement()`|`Int`, `Int64`|next value of a sequence|

A generator on a field of another type is E0406. A target that does not support a generator is E0421 at the field.

### 7.7 Argument values

**Binding.** Positional arguments come first and bind to parameters in declaration order; named arguments follow. A positional argument after a named one, a missing argument without a default, an unknown name and a parameter bound twice are E0403. A parameter of type `T?` MAY be omitted.

**Parameter types.** A parameter type is a built-in scalar, an enum, `List<...>` of these, or one of the meta-types below. Meta-types exist only in attribute declarations.

|Meta-type|Accepts|Available to|
|---|---|---|
|`FieldRef`|the name of a field of the annotated declaration (for an attribute on a field: the declaration that contains it)|core, plugins, users|
|`Number`|an int or float literal, checked against the type of the annotated field or parameter|core|
|`Value`|any value, checked against the type of the annotated field or parameter|core (`@default`)|

**Resolution.** A `value` is resolved by the type of the parameter it is bound to, never by guessing:

|Parameter type|Value written as|Example|
|---|---|---|
|scalar|a literal|`@length(max: 320)`|
|enum|a bare variant name of that enum|`@storage(columns)`, `onDelete: Cascade`, `@format(Email)`|
|`FieldRef`|a bare field name; an unknown field is E0408|`@index(fields: [email, createdAt])`|
|`Value`|a literal, a variant of the field's enum type, or a value generator|`@default(Member)`, `@default(now())`|
|`List<P>`|`[...]` of values of `P`|`fields: [userId]`|

Built-in enums used by core attributes are `StorageStrategy` (`json`, `columns`, `table`), `ReferentialAction` (`Cascade`, `Restrict`, `SetNull`) and `Format` (7.2).

**Literals against types.** A mismatch is E0403.

|Literal|Fits|
|---|---|
|int|`Int`, `Int64` (range-checked), `Float`, `Decimal`|
|float|`Float`, `Decimal`|
|string|`String`; `Uuid` in RFC 9562 text form; `DateTime` in RFC 3339 form; `Date` as `YYYY-MM-DD`|
|bool|`Bool`|
|`null`|only `@default(null)` on a nullable field|

`Bytes`, `Json`, collection, record and union fields cannot have a `@default` in v0.1.

---

## 8. Relations

A field whose type is a `model` (or `List<Model>`) is a relation field. A relation is a pair of relation fields, one in each model.

```
model Post {
  id: Uuid @primary @default(uuid())
  authorId: Uuid
  author: User @relation(fields: [authorId])
}

model User {
  id: Uuid @primary @default(uuid())
  posts: List<Post>
}
```

Rules:

- **Owning side.** Exactly one side of a relation has `@relation(fields: [...])`; it holds the foreign key (E0501). `references:` defaults to the target's primary key.
- **Back-relation.** The other side MUST be declared and has no `fields:` (E0504). Generators never invent relation fields.
- **Shape.** If the foreign-key fields are unique (a `@unique` field or a model-level `@unique` over exactly these fields), the relation is one-to-one and the back-relation is `Model?`. Otherwise it is one-to-many and the back-relation is `List<Model>`. Any other shape is E0505.
- **Keys.** `fields` and `references` MUST have the same length, and each pair MUST have the same type (E0220). Composite keys list several fields. `references` MUST be the target's primary key or a unique set of fields (E0506).
- **Optionality.** `User?` on the owning side means an optional relation. The relation is optional exactly when its foreign-key fields are nullable (E0503).
- **Names.** If two relations connect the same pair of models, each MUST have a name on both sides: `@relation(name: "authored", fields: [authorId])` and `@relation(name: "authored")` (E0502).
- **Self-relations** are allowed and follow the same rules; both fields are in the same model.
- **Many-to-many** needs an explicit join model with two one-to-many relations. `List<Tag>` on both sides is E0501. Implicit join tables are out of scope for v0.1.
- **Referential actions:** `@relation(fields: [...], onDelete: Cascade | Restrict | SetNull)`. The default is `Restrict`. `SetNull` requires an optional relation (E0507).

---

## 9. Composition

### 9.1 Mixins

`with A, B` copies the fields of mixins `A` and `B` into the declaration, in order, before its own fields.

```
mixin Timestamps { createdAt: DateTime @default(now()) }
mixin SoftDelete { deletedAt: DateTime? }

model Order with Timestamps, SoftDelete {
  id: Uuid @primary @default(uuid())
}
```

Rules:

- A mixin MAY use other mixins. Cycles are errors (E0601).
- A mixin reached by several paths (`with A, B` where both use `Base`) is copied once, at its first occurrence.
- Field names from different mixins MUST NOT conflict (E0602).
- A declaration MUST NOT redeclare a field it receives from a mixin (E0603). Overrides may be added later as `override field: Type`.
- Mixin fields keep their attributes. The IR records the mixin in each field's `origin`.
- Declaration-level attributes of a mixin are not copied to the declarations that use it.

### 9.2 Views

A `view` derives a new type from a `model`, `record` or another `view`. Operators apply left to right.

|Operator|Meaning|
|---|---|
|`without { a, b }`|remove fields|
|`only { a, b }`|keep only these fields|
|`+ { field }`|add declared fields|
|`partial`|make every field optional to provide; nullability is unchanged (5.3)|

```
view PublicUser = User without { passwordHash }
view CreateUser = User only { email, name }
view UpdateUser = CreateUser partial
view UserWithStats = PublicUser + { postCount: Int }
```

Rules:

- A view is a `record`-like type: no storage, no identity.
- **Relation fields are not part of a view.** When the source is a model, its relation fields (both sides) are dropped before any operator applies; foreign-key fields stay as ordinary scalar fields. Naming a relation field in an operator is E0610. Embedding related objects is planned as `include` ([[#14. Out of scope for v0.1]]).
- Naming a field that does not exist is an error (E0610). `+ { }` adding a field whose name already exists is E0613.
- **Attributes.** Storage attributes of the source fields are dropped: `@primary`, `@unique`, `@relation`, `@storage`, `@updated`, and `@default` with a value generator. Everything else is kept: validation attributes, `@default` with a literal or enum variant, `@sensitive`, `@doc`, user and plugin attributes. Declaration-level attributes of the source are not inherited.
- **Fields added with `+ { }` are declared, not computed.** The view states that the field exists; application code supplies its value.
- **Origin.** The IR records the source of the view, and for every field whether it is inherited (with the source declaration and field) or added. Generators use this to emit conversions from the source to the view.
- **Sensitive fields.** A view that contains a `@sensitive` field is an error (E0611) unless the view carries `@allowSensitive`. The check is transitive: a field is sensitive if it has `@sensitive`, or if its type is (or contains, through `List`, `Map` or a union) a record or view with a sensitive field.
- Views MUST NOT form cycles (E0612).

---

## 10. Modules and names

- Every `.linca` file under the schema root is a module. Its path is the file path relative to the root, with `/` replaced by `::` and the extension removed: `schema/shop/users.linca` → `shop::users`.
    
- Directory layout is a convention only; it has no other meaning.
    
- Imports:
    
    ```
    import shop::users                 // use as users::User
    import shop::users::{User, Role}   // use as User, Role
    ```
    
- A module imported without braces is referred to by its last path segment (`users::User`).
    
- Importing a module that does not exist is E0704. Importing an item that does not exist is E0705.
    
- Each module has **one namespace** for all its declarations: types, mixins, services, errors, attributes and attribute sets. Two declarations with the same name in one module are E0701. Fields and enum variants are scoped to their declaration, operations to their service, parameters to their operation.
    
- A name MUST resolve to exactly one thing: a local declaration, an imported item or a built-in. A local declaration and an imported item with the same name, two imported items with the same name, or two imported modules with the same last segment are E0702.
    
- Built-in names (scalar types, `List`, `Map`, built-in enums, core attributes) MUST NOT be redeclared (E0703).
    
- Import cycles between modules are allowed; declaration cycles are checked separately (mixins, views, attribute sets).
    

---

## 11. Semantic checks (summary)

The checker runs after parsing, in this order:

1. Module discovery and name resolution.
2. Expansion of attribute sets.
3. Mixin expansion and conflict checks.
4. View derivation, including the sensitive-field check.
5. Type checks of fields and attribute arguments.
6. Model-level checks: primary keys, relations, storage strategies.
7. Service checks: signatures, errors, the HTTP binding.
8. Target capability checks against configured plugins.

The IR is built only if there are no errors.

---

## 12. Diagnostics

Codes: `E` = error, `W` = warning. Ranges: 01xx lexical/syntax, 02xx types, 03xx declarations, 04xx attributes, 05xx relations, 06xx composition, 07xx modules, 08xx services.

Codes are stable. Once a version of this specification is released, a code is never reused for another meaning; a retired code stays retired.

|Code|Message|
|---|---|
|E0101|unexpected character|
|E0102|unterminated string or comment|
|E0103|unexpected token|
|E0104|keyword used as a declaration, variant, operation or module name|
|E0105|doc comment not attached to anything|
|E0106|separator between fields|
|E0201|unknown type|
|E0204|map key must be `String`|
|E0205|collection cannot be nullable|
|E0210|model used as field type in a record|
|E0211|mixin used as a type|
|E0212|view used as field type in a model|
|E0220|type mismatch (nominal types differ)|
|E0221|`type` base is not a built-in scalar|
|E0222|`Decimal` without `@decimal`|
|E0301|model has no primary key|
|E0302|model has more than one primary key|
|E0303|record has storage attributes|
|E0304|duplicate field|
|E0310|enum mixes string and int values|
|E0311|duplicate enum variant or value|
|E0312|int enum variant without a value|
|E0320|union member is not a record|
|E0321|union member does not contain required mixin fields|
|E0322|duplicate discriminator value|
|E0323|discriminator name is a field of a member|
|E0401|unknown attribute|
|E0402|attribute not allowed here|
|E0403|wrong attribute argument|
|E0404|non-repeatable attribute repeated|
|E0406|value generator not allowed for field type|
|E0407|namespaced attribute declared outside a plugin manifest|
|E0408|unknown field in attribute argument|
|E0409|duplicate rule name|
|E0410|attribute set cycle|
|E0420|storage strategy not supported by a target|
|E0421|value generator not supported by a target|
|E0422|construct not supported by a target|
|E0501|relation does not have exactly one owning side|
|E0502|ambiguous relations need names|
|E0503|relation and foreign key differ in optionality|
|E0504|missing back-relation|
|E0505|back-relation has the wrong shape|
|E0506|`references` is not a primary key or unique set|
|E0507|`SetNull` on a required relation|
|E0601|mixin cycle|
|E0602|field conflict between mixins|
|E0603|field redeclares a mixin field|
|E0610|view references unknown field|
|E0611|sensitive field exposed in a view|
|E0612|view cycle|
|E0613|view adds a field that already exists|
|E0701|duplicate declaration|
|E0702|ambiguous name|
|E0703|built-in name redeclared|
|E0704|unknown module|
|E0705|imported item does not exist|
|E0801|duplicate operation in a service|
|E0802|duplicate parameter in an operation|
|E0803|error used as a type|
|E0804|`throws` names something that is not an error|
|E0805|error listed twice in `throws`|
|E0806|model used in an operation signature|
|E0807|sensitive data in an operation result or in an error|
|E0808|error has a field named `error`|
|E0810|malformed path|
|E0811|more than one method attribute on an operation|
|E0812|path placeholder does not match a parameter|
|E0813|parameter cannot be placed there|
|E0814|duplicate route|
|E0815|`@readonly` on an operation that changes state|
|W0001|naming convention|
|W0401|attribute of an unconfigured plugin|
|W0405|pattern uses non-portable regex features|

Each diagnostic has a primary span, optional secondary spans and an optional fix suggestion.

---

## 13. Full example

```linca
// schema/shop/common.linca
mixin Timestamps {
  createdAt: DateTime @default(now())
  updatedAt: DateTime @updated
}

type UserId = Uuid

attribute email on field = [@format(Email), @length(max: 320)]

attribute id on field = [@primary, @default(uuid())]
```

```
// schema/shop/users.linca
import shop::common::{Timestamps, UserId, email, id}
import shop::orders::{Order}

enum Role { Admin Member }

model User with Timestamps {
  id: UserId @id
  email: String @email @unique
  passwordHash: String @sensitive
  role: Role @default(Member)
  orders: List<Order>
}

view PublicUser = User without { passwordHash }
```

```
// schema/shop/orders.linca
import shop::common::{Timestamps, UserId, id}
import shop::users::{User}

record Address {
  street: String
  city: String
}

mixin PaymentBase { amount: Decimal @decimal(12, 2) }

record CardPayment with PaymentBase { last4: String @length(4) }
record BankTransfer with PaymentBase { iban: String }

union Payment requires PaymentBase = CardPayment as "card" | BankTransfer as "bank_transfer"

@index(fields: [userId, createdAt])
model Order with Timestamps {
  id: Uuid @id
  userId: UserId   // Uuid here would be E0220
  user: User @relation(fields: [userId], onDelete: Cascade)
  shipping: Address @storage(columns)
  payment: Payment
  items: List<String>
}
```

A service over these models is shown in [[#17.9 Example]].

---

## 14. Out of scope for v0.1

- Generics (`Page<T>`), parameterized mixins and attribute sets.
- Events, streaming and operations that push data to the caller.
- Authentication and authorization of operations.
- Request and response bodies other than JSON: file upload, multipart, binary downloads.
- Response headers and cookies.
- Resource-style declarations that derive CRUD operations from a model.
- API versioning.
- Field overrides in mixins.
- Unions of models.
- Embedding related objects in views (`include`).
- Implicit many-to-many join tables.
- Overlay files for plugin attributes and for the HTTP binding (`extend ... for ...`).
- Computed fields and validation expressions (CEL may be considered later).
- Schema evolution and breaking-change rules.

## 15. Decisions and open questions

Decided in this draft:

- Separators: none. Fields are not separated by commas, `;` or significant newlines ([[#4. Grammar]]).
- `type` is a nominal newtype over a built-in scalar ([[#5.5 Domain types]]).
- The grammar accepts any `type_ref` as the base of `type`; the check reports E0221 for a base that is not a built-in scalar (ADR-0001).
- Repeatability is a property of the attribute declaration ([[#7.1 General rules]]).
- Attribute sets have no parameters in v0.1 ([[#7.4 User-defined attributes]]).
- Default generators are built-in markers, not functions ([[#7.6 Value generators]]).
- A parameter of an attribute declaration may be named like a keyword, as an argument and an operation parameter may ([[#3.3 Keywords]], ADR-0002).
- Declaration-level attributes are always written before the declaration, including `type`; attribute sets use brackets ([[#4. Grammar]]).
- A string literal cannot span lines; a raw line break inside it is E0102 ([[#3.4 Literals]], ADR-0003).
- Nullability and presence are separate properties; `partial` changes only presence ([[#5.3 Optionality]]).
- The back-relation is required, and generators never invent relation fields ([[#8. Relations]]).
- A union states required mixins with `requires`, not `with` ([[#6.5 `union`]]).
- The default discriminator field is `kind`; per-member values are set with `as` at the union ([[#6.5 `union`]]).
- Relation fields are dropped from views; fields added with `+ { }` are declared, not computed ([[#9.2 Views]]).
- A sensitive field in a view is an error, checked transitively ([[#9.2 Views]]).
- Argument values are resolved by parameter type ([[#7.7 Argument values]]).
- Attributes have four levels, and core attributes name the backend kinds that must honor them ([[#7.1 General rules]]).
- Target-specific representations are not part of this document; they belong to the plugin protocol specification.
- Contextual keywords without escaping ([[#3.3 Keywords]]).
- `Decimal` always requires `@decimal` ([[#5.1 Built-in scalar types]]).
- `uuid7()` as a separate generator ([[#7.6 Value generators]]).
- `@rule` is not carried into views ([[#7.2 Core attributes]]).
- Default referential action `Restrict` ([[#8. Relations]]).
- A mixin reached by several paths is copied once ([[#9.1 Mixins]]).
- An operation is described as a function; HTTP is bound to it with core attributes, not with grammar and not with plugin attributes ([[#17. Services]]).
- Errors are declarations listed after `throws`, not members of the result type ([[#17.4 Errors]]).
- An operation without a method attribute gets a binding derived by the language, so all plugins agree on its address ([[#17.6 Derived binding]]).
- Sensitive data cannot leave through an operation result unless the operation allows it ([[#17.8 Sensitive data]]).
- The names of a derived path are in kebab-case ([[#17.6 Derived binding]]).

Proposed in this draft, to confirm:

- A model cannot appear in an operation signature; a view is used instead ([[#17.3 Parameters and results]]).
- Parameters are placed by a default rule; `@query`, `@header` and `@body` override it ([[#17.5 HTTP binding]]).
- One body parameter of a record, union or view type is the whole body; any other set of body parameters forms an object ([[#17.5 HTTP binding]]).
- Default statuses: 200 with a result, 204 without, 400 for an error ([[#17.5 HTTP binding]]).
- An error is sent as its fields plus the member `error` with the error's name ([[#17.5 HTTP binding]]).
- A derived binding is always `POST`, also for a read-only operation ([[#17.6 Derived binding]]).
- An error cannot contain sensitive fields, and there is no way to allow it ([[#17.8 Sensitive data]]).

Open:

1. Unions of models and their storage. To research.
2. Presence of fields with a default in views. A field with `@default(Member)` could be optional in an input view (`CreateUser`) but should stay required in an output view (`PublicUser`). With operations the language can tell the two apart: a type reachable from a parameter is an input, a type reachable from a result is an output, and one type may be both. Whether such a field becomes optional in input positions, and how the IR says so, is not decided.
3. Casing of built-in enum variants. `StorageStrategy` uses lowercase (`json`, `columns`, `table`), while the naming convention and the other built-in enums use PascalCase. Pick one.
4. Names on the wire. Section 17.5 sends parameters under their names as written. The names of record fields in JSON are left to the plugin protocol, and the plugins on both sides of one API have to agree on them.
5. Pages of results. `Page<T>` needs generics; until then a page is a record per element type (`UserPage`).
6. Authentication. An operation cannot yet say that it needs an identified caller, or which rights it needs.

---

## 16. Capability-gated constructs

The constructs below are valid Linca but not every target can represent them. The core checks them against the capabilities of every configured plugin before generation (step 8 in Section 11). The list is closed in v0.1; capability names and the way plugins declare them are defined by the plugin protocol specification.

|Construct|Where defined|Diagnostic|
|---|---|---|
|Storage strategy `json`|7.3|E0420|
|Storage strategy `columns`|7.3|E0420|
|Storage strategy `table`|7.3|E0420|
|Value generator `uuid()`|7.6|E0421|
|Value generator `uuid7()`|7.6|E0421|
|Value generator `now()`|7.6|E0421|
|Value generator `autoincrement()`|7.6|E0421|
|Enum with int values|6.4|E0422|
|Field of type `Json`|5.1|E0422|
|Field of type `Map<String, T>` in a model|5.2|E0422|
|Composite primary or foreign key|6.1, 8|E0422|

Everything not listed here MUST be supported by every plugin of the backend kinds that the construct concerns.

---

## 17. Services

### 17.1 Overview

A `service` is a named group of operations. An operation is described the way a function is: a name, parameters, a result and the errors it may raise. This description says nothing about a transport.

The binding to HTTP is added with core attributes ([[#17.5 HTTP binding]]). An operation without them gets a binding derived by the language ([[#17.6 Derived binding]]). Every operation therefore has exactly one HTTP address, and the plugin that generates a server and the plugin that generates a client always agree on it. A plugin for another transport ignores the binding and uses the neutral description.

```
@route("/users")
service Users {
  @get("/{id}")
  getUser(id: UserId) -> PublicUser throws NotFound
}
```

### 17.2 Operations

- A service contains zero or more operations. A service is not a type.
- Operation names MUST be unique within a service (E0801). A keyword MUST NOT be used as an operation name (E0104).
- An operation has zero or more parameters, at most one result and zero or more errors.
- An operation without `->` returns nothing.
- `throws` lists the errors the operation may raise. Every item MUST be an `error` declaration (E0804) and MUST be listed once (E0805). The order has no meaning.
- Operations are not inherited, overloaded or generic.

### 17.3 Parameters and results

- Parameter names MUST be unique within an operation (E0802). A keyword MAY be used as a parameter name, as for fields.
- A parameter type or a result type is any type the last column of the table in [[#5.6 Where types may be used]] allows: a scalar, an enum, a domain type, a record, a union, a view, or a `List` or `Map` of these.
- **A model MUST NOT appear in an operation signature** (E0806), directly or inside `List` or `Map`. What an API accepts and returns is stated with a view. A view without operators is enough: `view UserOut = User`.
- **Nullability** works as for fields: `T?` means the value may be null. A result of type `T?` may be null.
- **Presence.** A parameter is required unless it has `@default`; a parameter with `@default` is optional, and the default is used when it is omitted. The default MUST be a literal or an enum variant; a value generator is E0406.
- Parameters MAY carry the validation attributes `@min`, `@max`, `@length`, `@pattern` and `@format`, and `@doc`. They are checked as on a field of the same type.

### 17.4 Errors

An `error` declares one way an operation may fail, with the data that comes with the failure.

```
@status(404)
error NotFound { resource: String }

error Unauthorized {}
```

- An error has fields, like a record, and MAY use mixins with `with`. The rules of a record apply to its fields: no storage attributes (E0303), no model as a field type (E0210), unique names (E0304).
- An error is **not a type**. It MUST NOT be used as a field, parameter or result type, or as the source of a view (E0803), or as a union member (E0320). It is named only after `throws`.
- Errors are nominal. Two errors with the same fields are different errors.
- An error MUST NOT have a field named `error` (E0808); the name is used on the wire (17.5).
- Only declared errors are part of the language. A request that does not pass validation, a transport failure and an unexpected fault of the server are not described in the schema; their form is defined by the target framework.

### 17.5 HTTP binding

**Attributes.**

|Attribute|Signature|Target|
|---|---|---|
|`@route`|`@route(path: String)`|service|
|`@get`, `@post`, `@put`, `@patch`, `@delete`|`@get(path: String = "")`|operation|
|`@query`|`@query`|param|
|`@header`|`@header(name: String)`|param|
|`@body`|`@body`|param|
|`@status`|`@status(code: Int)`|operation, error|

An operation MUST NOT have more than one of `@get`, `@post`, `@put`, `@patch`, `@delete` (E0811).

**Paths.** A path is a string of zero or more segments, each written after a `/`:

```
path     = { "/" segment } ;
segment  = literal | "{" ident "}" ;
literal  = pchar { pchar } ;      (* letters, digits and - . _ ~ *)
```

- A placeholder `{name}` always takes a whole segment. A path has no trailing `/`, no empty segment and no query string. Anything else is E0810.
- The **full path** of an operation is the `@route` path of its service followed by the path of its method attribute. A service without `@route` has an empty prefix. An empty full path means `/`.
- Every placeholder of the full path MUST name a parameter of the operation, and a parameter MUST NOT be named by two placeholders (E0812). A placeholder in `@route` therefore requires that parameter in every operation of the service that has a method attribute. An operation that lacks it is reported at the name of the operation, once for each such operation, with the `@route` attribute as a secondary span; the same holds for an operation with a derived binding ([[#17.6 Derived binding]]).
- Two operations of the schema MUST NOT have the same method and the same full path (E0814). Paths are compared segment by segment, and two placeholders are equal whatever their names. `/users/{id}` and `/users/me` are different paths; when a request matches both, the literal segment wins.

**Placement.** Every parameter is sent in exactly one place:

1. A parameter named by a placeholder is sent in the **path**.
2. A parameter with `@query`, `@header` or `@body` is sent there.
3. Every other parameter is sent in the **query** if the method is `GET` or `DELETE`, and in the **body** otherwise.

A parameter MUST NOT have more than one placement attribute, and a parameter named by a placeholder MUST NOT have any (E0813). An operation with `GET` or `DELETE` MUST NOT have body parameters (E0813).

|Placement|Allowed parameter types|Null and default|
|---|---|---|
|path|a scalar except `Bytes` and `Json`, an enum, a domain type|MUST NOT be nullable and MUST NOT have `@default`|
|query|the same, or a `List` of them with elements that are not nullable|a null value is sent by omitting the parameter; a list repeats the key|
|header|as for path|a null value is sent by omitting the header|
|body|any type allowed in a signature|—|

A type or a default the placement does not allow is E0813. A query parameter is sent under its name as written; a header under the name given in `@header`.

**Body.** Bodies are JSON.

- If exactly one parameter is placed in the body and its type is a record, a union or a view, the body is that value.
- Otherwise the body is an object with one member for each body parameter, named as the parameter is written.
- An operation without body parameters has no request body.

The response body is the result. An operation without a result has an empty response body.

**Status.**

- `@status` on an operation sets the status of a successful response and MUST be in the range 200–299 (E0403). The default is 200 for an operation with a result and 204 for an operation without one. An operation with a result MUST NOT have status 204 (E0403).
- `@status` on an error sets the status of the response that carries it and MUST be in the range 400–599 (E0403). The default is 400. An error with a status of 400–499 is a _client_ error, one with 500–599 a _server_ error; plugins for other transports use this category.

**Errors on the wire.** An error is sent with its status and with an object as the body: the fields of the error, and the member `error` whose value is the name of the error as written.

```json
{ "error": "NotFound", "resource": "user" }
```

Several errors of one operation MAY have the same status; the member `error` tells them apart.

### 17.6 Derived binding

An operation without a method attribute is bound by this rule:

- The method is `POST`.
- The full path is the prefix followed by `/` and the name of the operation in kebab-case. The prefix is the `@route` path of the service, or `/` and the name of the service in kebab-case if the service has no `@route`. A `@route` path with a placeholder requires the named parameter (E0812), as for any other operation.
- The kebab-case form of a name is its words, lowercase, joined with `-`. A name is split into words by the rule of Section 4.1 of [[IR Specification]]: `countOrders` gives `count-orders`, `OrderItems` gives `order-items`. A `@route` path is used as written and is never changed.
- Parameters are placed by the rule of 17.5, so every parameter without a placement attribute goes to the body.
- Status and errors work as in 17.5.

```
service Billing {
  refund(orderId: OrderId, amount: Money)    // POST /billing/refund, body { "orderId": …, "amount": … }
}
```

The rule is part of the language, not of a plugin, and it cannot be configured. An operation that needs another address gets a method attribute. A schema without a single HTTP attribute is a complete description of an RPC-style API.

### 17.7 Read-only operations

An operation is **read-only** if it changes no state and may be repeated or cached. The IR records this for every operation as a boolean.

- An operation with `@get` is read-only.
- An operation with `@readonly` is read-only. The attribute is meant for operations with a derived binding; on an operation with `@get` it changes nothing.
- `@readonly` together with `@post`, `@put`, `@patch` or `@delete` is E0815.

Plugins use the property where the target has such a distinction: a query or a mutation, a cached or an uncached call. It does not change the derived binding.

### 17.8 Sensitive data

Sensitivity is defined in [[#9.2 Views]]: a type is sensitive if it is, or contains, a record or view with a sensitive field.

- A **result** MUST NOT be of a sensitive type unless the operation carries `@allowSensitive` (E0807). A view with `@allowSensitive` is still sensitive, so returning it needs the attribute on the operation as well.
- An **error** MUST NOT have a sensitive field (E0807). There is no way to allow it.
- **Parameters** MAY be of a sensitive type: a password has to be accepted somewhere.

### 17.9 Example

```
// schema/shop/api.linca
import shop::common::{UserId}
import shop::users::{User, PublicUser, Role}

view CreateUser = User only { email, role }
view UpdateUser = CreateUser partial

record UserPage {
  items: List<PublicUser>
  total: Int
}

@status(404)
error NotFound { resource: String }

@status(409)
error EmailTaken { email: String }

@route("/users")
service Users {
  @get
  listUsers(
    role: Role?,
    page: Int @default(1) @min(1),
    size: Int @default(20) @max(100),
  ) -> UserPage

  @get("/{id}")
  getUser(id: UserId) -> PublicUser throws NotFound

  @post @status(201)
  createUser(input: CreateUser) -> PublicUser throws EmailTaken

  @patch("/{id}")
  updateUser(id: UserId, input: UpdateUser) -> PublicUser throws NotFound, EmailTaken

  @delete("/{id}")
  deleteUser(id: UserId) throws NotFound

  /// No method attribute: the binding is derived.
  resetPassword(email: String @format(Email))
}
```

What the binding gives:

|Operation|Request|Success|Errors|
|---|---|---|---|
|`listUsers`|`GET /users?role=…&page=…&size=…`|200, `UserPage`|—|
|`getUser`|`GET /users/{id}`|200, `PublicUser`|404 `NotFound`|
|`createUser`|`POST /users`, body is a `CreateUser`|201, `PublicUser`|409 `EmailTaken`|
|`updateUser`|`PATCH /users/{id}`, body is an `UpdateUser`|200, `PublicUser`|404 `NotFound`, 409 `EmailTaken`|
|`deleteUser`|`DELETE /users/{id}`|204|404 `NotFound`|
|`resetPassword`|`POST /users/reset-password`, body `{ "email": … }`|204|—|

`listUsers` and `getUser` are read-only.
