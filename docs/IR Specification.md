---

## title: Linca IR Specification version: 0.1 status: draft date: 2026-11-16 tags: [linca, spec, ir]

# Linca IR Specification

**Version:** 0.1 (draft) **Status:** for discussion **Scope:** the intermediate representation only. The language is specified in [[Specification]]; the plugin protocol and the CLI are specified separately.

The normative definition of the format is the JSON Schema `schema/linca-ir.schema.json` that accompanies this document. Where this text and the schema disagree, the schema wins and this text is to be fixed.

---

## 1. Overview

The IR is the output of the Linca compiler and the only input of a plugin. It describes a whole, valid schema as one JSON document. A plugin reads it, walks it and prints code; it does not need the source files, the grammar or the rules of the language.

Design rules:

1. **Fully expanded.** Everything the language derives is already derived: mixin fields are copied into the declarations that use them, views are materialized into field lists, attribute sets are expanded, the HTTP binding of every operation is resolved, defaults of the language are filled in. A plugin computes nothing about the language.
2. **Flat, linked by id.** Declarations are one flat list. A reference to a declaration is its id, a string. There is no nesting of one declaration inside another, so cyclic relations are not a problem.
3. **Order is data.** Everything ordered is an array. A JSON object is used only for a record with a fixed set of keys.
4. **Explicit.** Every key of a node is always present. A missing value is `null`; an empty collection is `[]`. A reader never has to ask whether a key exists.
5. **Tagged.** Every node that has variants carries `kind`.
6. **One fact, once.** A fact appears in one place. A core attribute that became a property is not repeated in `attributes`.
7. **Deterministic.** The same schema always gives the same bytes ([[#9. Canonical form]]).

The IR is built only for a schema without errors. Diagnostics are not part of it.

---

## 2. Conventions

### 2.1 Encoding

- JSON (RFC 8259), UTF-8, no BOM.
- Keys are camelCase ASCII.
- **Numbers.** JSON numbers are used only for counts and positions: lengths, precision, scale, lines, columns. Every numeric value that comes from the schema (a default, a bound of `@min`/`@max`, an enum value) is a **string**, because `Int64` and `Decimal` do not survive a JSON number in some languages.
- Plain strings are used for names and ids; there are no other string encodings to decode.

### 2.2 Identifiers

- A **declaration id** is its module path, `::`, and its name: `shop::users::User`. It is unique in the document.
- A **module id** is the module path: `shop::users`.
- A **field** is referred to by its source name within its declaration: `"userId"`.
- An **enum variant** is referred to by the enum id and the variant's source name.
- An **operation** is referred to by its source name within its service, a **parameter** by its source name within its operation.

### 2.3 `null` and empty

- A key whose value does not apply is `null`: a field without a default has `"default": null`.
- A collection without elements is `[]`, never `null`.
- A boolean is always `true` or `false`.

### 2.4 Unknown content

- A reader MUST ignore keys it does not know.
- A reader MUST ignore a declaration whose `kind` it does not know.
- A reader MUST fail on an unknown `kind` anywhere else (a type, a value, an origin), because skipping it would silently change the meaning.

---

## 3. Document

|Key|Type|Meaning|
|---|---|---|
|`irVersion`|string|version of this format, `"0.1"` ([[#10. Versioning]])|
|`languageVersion`|string|version of the language specification the schema was checked against|
|`modules`|array of Module|every module of the schema, sorted by `id`|
|`declarations`|array of Declaration|every declaration, in module order and then in source order|

**Module:**

|Key|Type|Meaning|
|---|---|---|
|`id`|string|module path, e.g. `shop::users`|
|`file`|string|path of the source file relative to the schema root, with `/` separators|

A schema of many files gives **one** document. The file a declaration came from is kept as data (`module`, `span`). How many files a plugin writes, and how it groups declarations into them, is the plugin's choice.

---

## 4. Building blocks

### 4.1 Name

Every name written by the user is given both as written and as words.

|Key|Type|Meaning|
|---|---|---|
|`source`|string|the name exactly as written in the schema|
|`words`|array of string|the name split into lowercase words|

A plugin builds the name for its target from `words` (`created_at`, `CreatedAt`, `CREATED_AT`), so every plugin splits a name the same way. A plugin that needs the exact source spelling uses `source`.

Splitting rule, applied to `source`:

1. Split at every `_`.
2. Inside each part, start a new word before an uppercase letter that follows a lowercase letter or a digit.
3. In a run of uppercase letters followed by a lowercase letter, the last uppercase letter starts a new word.
4. A digit never starts a word; it stays with the letters before it.
5. Lowercase every word.

|`source`|`words`|
|---|---|
|`createdAt`|`created`, `at`|
|`OrderItem`|`order`, `item`|
|`order_item`|`order`, `item`|
|`userID`|`user`, `id`|
|`HTTPServer`|`http`, `server`|
|`ipv4Address`|`ipv4`, `address`|
|`last4`|`last4`|

### 4.2 Span

Where an item is written in the source. A plugin uses it to point its own messages at the `.linca` file. 

|Key|Type|Meaning|
|---|---|---|
|`file`|string|as in Module|
|`line`|integer|1-based line|
|`column`|integer|1-based column, counted in characters|

The span of a declaration, field, relation or variant is the position of its **name**. The span of an attribute is the position of its `@`. A field copied from a mixin, or inherited by a view, keeps the span of the place where it is written.

### 4.3 TypeRef

The type of a field, a parameter or an operation result. One of four kinds; every kind has `nullable`.

|`kind`|Other keys|Meaning|
|---|---|---|
|`scalar`|`name`, `nullable`|a built-in scalar: `String`, `Int`, `Int64`, `Float`, `Decimal`, `Bool`, `Uuid`, `DateTime`, `Date`, `Bytes`, `Json`|
|`ref`|`id`, `nullable`|a declaration: an enum, record, union, view or domain type|
|`list`|`element`, `nullable`|`List<T>`; `element` is a TypeRef|
|`map`|`value`, `nullable`|`Map<String, T>`; `value` is a TypeRef; keys are always strings|

`nullable` is `true` exactly where the schema has `?`. For `list` and `map` it is always `false`, because the language does not allow a nullable collection; the elements carry their own `nullable`.

A TypeRef never points at a model or an error. A field whose type is a model is a relation and is listed in `relations` ([[#7. Relations]]). An error is referred to by its id in the `errors` of an operation ([[#6.9 Service]]).

### 4.4 Value

A value written in the schema: a default of a field or parameter, or an attribute argument.

|`kind`|Other keys|Meaning|
|---|---|---|
|`string`|`value`|a string literal|
|`int`|`value`|an integer literal, as a decimal string|
|`float`|`value`|a float literal, as a string|
|`bool`|`value`|`true` or `false`|
|`null`|—|the literal `null`|
|`variant`|`enum`, `name`|an enum variant: enum id and the variant's source name|
|`generated`|`generator`|a value generator: `uuid`, `uuid7`, `now`, `autoincrement`|
|`field`|`name`|a field of the annotated declaration (a `FieldRef` argument)|
|`list`|`items`|a list of values|

A literal is given as written, after the compiler has checked that it fits the field. A default of `"2026-01-01"` on a `Date` field is `{ "kind": "string", "value": "2026-01-01" }`; the field's type tells the plugin how to read it.

### 4.5 Constraints

The validation attributes of the language, as properties. Every key is always present.

|Key|Type|From|Meaning|
|---|---|---|---|
|`min`|string or null|`@min`|inclusive lower bound|
|`max`|string or null|`@max`|inclusive upper bound|
|`minLength`|integer or null|`@length`|minimum length|
|`maxLength`|integer or null|`@length`|maximum length|
|`pattern`|string or null|`@pattern`|regular expression matching the whole value|
|`format`|string or null|`@format`|`Email`, `Uri`, `Hostname`, `Ipv4`, `Ipv6`|
|`decimal`|object or null|`@decimal`|`{ "precision": 12, "scale": 2 }`|

- `@length(4)` gives `minLength` and `maxLength` of `4`.
- Length is counted in characters for `String`, bytes for `Bytes` and elements for a list.
- The constraints of a field are **effective**: they already include the constraints of the field's domain type. A plugin that only needs to validate a field reads the field alone. A plugin that emits a separate type for the domain type finds the type's own constraints on that declaration ([[#6.6 Domain type]]).

### 4.6 Origin

Where a field came from.

|`kind`|Other keys|Meaning|
|---|---|---|
|`own`|—|written in this declaration|
|`mixin`|`id`|copied from this mixin (the mixin where the field is written)|
|`inherited`|`declaration`, `field`|a view field taken from the view's source: source declaration id and field name|
|`added`|—|a view field added with `+ { }`|

`own` and `mixin` occur in models, records, mixins and errors; `inherited` and `added` occur in views. A view field inherited from a model field that itself came from a mixin has `inherited`; the mixin is found by following `declaration` and `field`.

### 4.7 Attribute

Only **user** and **plugin** attributes appear as Attribute nodes. Core attributes are properties of their node and are never listed here.

|Key|Type|Meaning|
|---|---|---|
|`id`|string or null|declaration id of a user attribute, e.g. `shop::common::pii`; `null` for a plugin attribute|
|`plugin`|string or null|namespace of a plugin attribute, e.g. `prisma`; `null` for a user attribute|
|`name`|string|attribute name without namespace: `pii`, `map`|
|`args`|array of `{ name, value }`|arguments|
|`set`|string or null|id of the attribute set this use was expanded from, or `null`|
|`span`|Span|position of the `@`|

- For a user attribute and for an attribute of a configured plugin, `args` lists **every** parameter in declaration order, with defaults filled in; `name` is the parameter name.
- For an attribute of a plugin that is not configured (W0401 in the language), the arguments are not checked: `args` lists them as written, and `name` is `null` for a positional argument.
- Attribute declarations and attribute sets themselves are not part of the IR.

---

## 5. Field

A data field of a model, record, mixin, view or error. Relation fields are not Field nodes ([[#7. Relations]]).

|Key|Type|Meaning|
|---|---|---|
|`name`|Name|field name|
|`doc`|string or null|documentation (`///` or `@doc`)|
|`type`|TypeRef|field type|
|`presence`|`"required"` or `"optional"`|whether the field must be provided; `optional` only after `partial`|
|`default`|Value or null|`@default`|
|`autoUpdated`|boolean|`@updated`|
|`constraints`|Constraints|effective validation constraints|
|`sensitive`|boolean|whether the field is sensitive|
|`storage`|`"json"`, `"columns"`, `"table"` or null|storage strategy|
|`origin`|Origin|where the field came from|
|`attributes`|array of Attribute|user and plugin attributes|
|`span`|Span|position of the name|

Notes:

- **Nullability and presence are separate**, as in the language: `type.nullable` says whether the value may be null, `presence` says whether the field may be omitted.
- **`sensitive`** is effective: `true` if the field has `@sensitive`, or if its type is or contains a record or view with a sensitive field.
- **`storage`** is set for a model field whose type is a record, a union or a collection; the default `json` is already filled in. It is `null` for every other field and for every field outside a model.
- **In a view**, storage facts are already removed: `autoUpdated` is `false`, `storage` is `null`, and a `default` that was a value generator is `null`. A literal or variant default is kept.

Example, the `email` field of `User`:

```json
{
  "name": {
    "source": "email",
    "words": [
      "email"
    ]
  },
  "doc": null,
  "type": {
    "kind": "scalar",
    "name": "String",
    "nullable": false
  },
  "presence": "required",
  "default": null,
  "autoUpdated": false,
  "constraints": {
    "min": null,
    "max": null,
    "minLength": null,
    "maxLength": 320,
    "pattern": null,
    "format": "Email",
    "decimal": null
  },
  "sensitive": false,
  "storage": null,
  "origin": {
    "kind": "own"
  },
  "attributes": [],
  "span": {
    "file": "shop/users.linca",
    "line": 9,
    "column": 3
  }
}
```

---

## 6. Declarations

Every declaration starts with the same five keys and ends with the same two.

| Key          | Type               | Meaning                                                     |
| ------------ | ------------------ | ----------------------------------------------------------- |
| `kind`       | string             | `model`, `record`, `view`, `enum`, `union`, `type`, `mixin`, `error`, `service` |
| `id`         | string             | declaration id                                              |
| `module`     | string             | module id                                                   |
| `name`       | Name               | declaration name                                            |
| `doc`        | string or null     | documentation                                               |
| …            | …                  | keys of the kind, below                                     |
| `attributes` | array of Attribute | user and plugin attributes on the declaration               |
| `span`       | Span               | position of the name                                        |
|              |                    |                                                             |

### 6.1 Model

|Key|Type|Meaning|
|---|---|---|
|`mixins`|array of id|every mixin whose fields the model contains, in expansion order, each once|
|`fields`|array of Field|data fields in order: mixin fields first, then own fields|
|`relations`|array of Relation|relation fields, in source order|
|`primaryKey`|array of field name|fields of the primary key, one or more|
|`uniques`|array of array of field name|uniqueness constraints: field-level `@unique` first, in field order, then model-level ones in source order|
|`indexes`|array of `{ fields }`|non-unique indexes, in source order|
|`rules`|array of string|names of `@rule`, in source order|

A field is not marked as primary or unique itself; `primaryKey` and `uniques` are the only place for these facts.

### 6.2 Record

|Key|Type|Meaning|
|---|---|---|
|`mixins`|array of id|as in Model|
|`fields`|array of Field|as in Model|
|`rules`|array of string|as in Model|

### 6.3 View

|Key|Type|Meaning|
|---|---|---|
|`source`|id|the model, record or view the view derives from|
|`allowSensitive`|boolean|`@allowSensitive`|
|`fields`|array of Field|the resulting fields, in order|
|`rules`|array of string|rules declared on the view itself|

The view operators (`without`, `only`, `+`, `partial`) are not in the IR; only their result is. What a plugin needs for a conversion from the source to the view is in each field's `origin`: an `inherited` field is copied from the source, an `added` field has to be supplied by application code.

A field of a view, `role` of `PublicUser`:

```json
{
  "name": {
    "source": "role",
    "words": [
      "role"
    ]
  },
  "doc": null,
  "type": {
    "kind": "ref",
    "id": "shop::users::Role",
    "nullable": false
  },
  "presence": "required",
  "default": {
    "kind": "variant",
    "enum": "shop::users::Role",
    "name": "Member"
  },
  "autoUpdated": false,
  "constraints": {
    "min": null,
    "max": null,
    "minLength": null,
    "maxLength": null,
    "pattern": null,
    "format": null,
    "decimal": null
  },
  "sensitive": false,
  "storage": null,
  "origin": {
    "kind": "inherited",
    "declaration": "shop::users::User",
    "field": "role"
  },
  "attributes": [],
  "span": {
    "file": "shop/users.linca",
    "line": 11,
    "column": 3
  }
}
```

### 6.4 Enum

|Key|Type|Meaning|
|---|---|---|
|`valueType`|`"string"` or `"int"`|type of the stored values|
|`variants`|array of Variant|variants in source order|

**Variant:**

|Key|Type|Meaning|
|---|---|---|
|`name`|Name|variant name|
|`doc`|string or null|documentation|
|`value`|string|the stored and transmitted value: the explicit value, or the name if none is given; an int value is a decimal string|
|`attributes`|array of Attribute|user and plugin attributes|
|`span`|Span|position of the name|

### 6.5 Union

|Key|Type|Meaning|
|---|---|---|
|`discriminator`|string|name of the discriminator field; the default `kind` is already filled in|
|`members`|array of `{ record, value }`|member record id and its discriminator value, in source order|
|`requires`|array of id|mixins every member is required to contain|

Example:

```json
{
  "kind": "union",
  "id": "shop::orders::Payment",
  "module": "shop::orders",
  "name": {
    "source": "Payment",
    "words": [
      "payment"
    ]
  },
  "doc": null,
  "discriminator": "kind",
  "members": [
    {
      "record": "shop::orders::CardPayment",
      "value": "card"
    },
    {
      "record": "shop::orders::BankTransfer",
      "value": "bank_transfer"
    }
  ],
  "requires": [
    "shop::orders::PaymentBase"
  ],
  "attributes": [],
  "span": {
    "file": "shop/orders.linca",
    "line": 15,
    "column": 7
  }
}
```

### 6.6 Domain type

`kind` is `"type"`.

|Key|Type|Meaning|
|---|---|---|
|`base`|string|the built-in scalar the type is based on|
|`constraints`|Constraints|constraints declared on the type|

### 6.7 Mixin

A mixin stays in the IR as a declaration so a plugin can emit a shared interface or base type. Its fields are **also** already copied into every declaration that uses it.

|Key|Type|Meaning|
|---|---|---|
|`mixins`|array of id|mixins this mixin contains, in expansion order|
|`fields`|array of Field|data fields, including those from other mixins|

A plugin that has no use for mixins ignores these declarations and loses nothing.

### 6.8 Error

|Key|Type|Meaning|
|---|---|---|
|`mixins`|array of id|as in Model|
|`fields`|array of Field|as in Model|
|`category`|`"client"` or `"server"`|who is at fault; derived from `status`|
|`http`|`{ status }`|`status` is the integer HTTP status of the response that carries the error; the default `400` is already filled in|

`category` is the transport-neutral fact; a plugin for a transport other than HTTP reads it and ignores `http`. On the wire the body of an error is its fields plus the member `error` with the source name of the error; this is fixed by the language and is not repeated in the IR.

Example, `NotFound`:

```json
{
  "kind": "error",
  "id": "shop::api::NotFound",
  "module": "shop::api",
  "name": {
    "source": "NotFound",
    "words": [
      "not",
      "found"
    ]
  },
  "doc": null,
  "mixins": [],
  "fields": [
    {
      "name": {
        "source": "resource",
        "words": [
          "resource"
        ]
      },
      "doc": null,
      "type": {
        "kind": "scalar",
        "name": "String",
        "nullable": false
      },
      "presence": "required",
      "default": null,
      "autoUpdated": false,
      "constraints": {
        "min": null,
        "max": null,
        "minLength": null,
        "maxLength": null,
        "pattern": null,
        "format": null,
        "decimal": null
      },
      "sensitive": false,
      "storage": null,
      "origin": {
        "kind": "own"
      },
      "attributes": [],
      "span": {
        "file": "shop/api.linca",
        "line": 14,
        "column": 18
      }
    }
  ],
  "category": "client",
  "http": {
    "status": 404
  },
  "attributes": [],
  "span": {
    "file": "shop/api.linca",
    "line": 14,
    "column": 7
  }
}
```

### 6.9 Service

|Key|Type|Meaning|
|---|---|---|
|`operations`|array of Operation|operations in source order|

Everything about HTTP is under a key named `http`: on the operation, on each parameter and on each error. The other keys describe the operation without a transport. A plugin for HTTP reads both; a plugin for another transport ignores every `http` key and loses nothing it needs.

**Operation:**

|Key|Type|Meaning|
|---|---|---|
|`name`|Name|operation name|
|`doc`|string or null|documentation|
|`params`|array of Param|parameters in source order|
|`result`|TypeRef or null|result type; `null` if the operation returns nothing|
|`errors`|array of id|errors of `throws`, in source order|
|`readonly`|boolean|whether the operation changes no state: `true` for `@get` and for `@readonly`|
|`allowSensitive`|boolean|`@allowSensitive`|
|`http`|HttpBinding|the HTTP binding, written or derived|
|`attributes`|array of Attribute|user and plugin attributes|
|`span`|Span|position of the name|

**Param:**

|Key|Type|Meaning|
|---|---|---|
|`name`|Name|parameter name|
|`doc`|string or null|documentation (`@doc`)|
|`type`|TypeRef|parameter type|
|`presence`|`"required"` or `"optional"`|`optional` exactly when the parameter has a default|
|`default`|Value or null|`@default`; never a `generated` value|
|`constraints`|Constraints|effective validation constraints, as for a field|
|`sensitive`|boolean|whether the type is or contains a record or view with a sensitive field|
|`http`|ParamBinding|where the parameter is sent|
|`attributes`|array of Attribute|user and plugin attributes|
|`span`|Span|position of the name|

**HttpBinding:**

|Key|Type|Meaning|
|---|---|---|
|`method`|`"GET"`, `"POST"`, `"PUT"`, `"PATCH"` or `"DELETE"`|HTTP method|
|`path`|array of Segment|the full path, with the `@route` prefix of the service already included; `[]` is the path `/`|
|`derived`|boolean|`true` if the operation has no method attribute and the binding was derived by the language; the derived segments are already in kebab-case|
|`status`|integer|status of a successful response; the default is already filled in|
|`body`|`"none"`, `"value"` or `"object"`|shape of the request body|

`body` is `value` when one parameter is the whole body, `object` when the body is an object with one member for each body parameter, and `none` when no parameter is sent in the body.

**Segment**, one of two kinds:

|`kind`|Other keys|Meaning|
|---|---|---|
|`literal`|`value`|a fixed segment, without `/`|
|`param`|`name`|a placeholder; `name` is the source name of a parameter of the operation|

A plugin prints the path in the form of its target (`/users/{id}`, `/users/:id`) from the segments and never parses a path string.

**ParamBinding:**

|Key|Type|Meaning|
|---|---|---|
|`in`|`"path"`, `"query"`, `"header"` or `"body"`|where the parameter is sent, with the default placement already applied|
|`name`|string|the name on the wire: the header name for `header`, the source name of the parameter otherwise|

For a `body` parameter `name` is the member name when the body is an `object`; it is not used when the body is a `value`.

Example, `getUser` of the service `Users`:

```json
{
  "name": {
    "source": "getUser",
    "words": [
      "get",
      "user"
    ]
  },
  "doc": null,
  "params": [
    {
      "name": {
        "source": "id",
        "words": [
          "id"
        ]
      },
      "doc": null,
      "type": {
        "kind": "ref",
        "id": "shop::common::UserId",
        "nullable": false
      },
      "presence": "required",
      "default": null,
      "constraints": {
        "min": null,
        "max": null,
        "minLength": null,
        "maxLength": null,
        "pattern": null,
        "format": null,
        "decimal": null
      },
      "sensitive": false,
      "http": {
        "in": "path",
        "name": "id"
      },
      "attributes": [],
      "span": {
        "file": "shop/api.linca",
        "line": 29,
        "column": 11
      }
    }
  ],
  "result": {
    "kind": "ref",
    "id": "shop::users::PublicUser",
    "nullable": false
  },
  "errors": [
    "shop::api::NotFound"
  ],
  "readonly": true,
  "allowSensitive": false,
  "http": {
    "method": "GET",
    "path": [
      {
        "kind": "literal",
        "value": "users"
      },
      {
        "kind": "param",
        "name": "id"
      }
    ],
    "derived": false,
    "status": 200,
    "body": "none"
  },
  "attributes": [],
  "span": {
    "file": "shop/api.linca",
    "line": 29,
    "column": 3
  }
}
```

The operation `resetPassword` of the same service has no method attribute. Its binding is derived, and the IR gives it in the same form:

```json
{
  "method": "POST",
  "path": [
    {
      "kind": "literal",
      "value": "users"
    },
    {
      "kind": "literal",
      "value": "reset-password"
    }
  ],
  "derived": true,
  "status": 204,
  "body": "object"
}
```

---

## 7. Relations

A relation is described from both ends: each of the two models lists its own end in `relations`. Relation fields are kept apart from `fields`, so a plugin that only needs data shapes never sees them.

|Key|Type|Meaning|
|---|---|---|
|`name`|Name|name of the relation field|
|`doc`|string or null|documentation|
|`target`|id|the model on the other end|
|`cardinality`|`"one"` or `"many"`|how many target records this end holds|
|`nullable`|boolean|for `one`: whether the target may be absent; always `false` for `many`|
|`owning`|boolean|whether this end holds the foreign key|
|`fields`|array of field name|foreign-key fields of this model; `[]` on the non-owning end|
|`references`|array of field name|referenced fields of the target, in the same order; `[]` on the non-owning end|
|`onDelete`|`"Cascade"`, `"Restrict"`, `"SetNull"` or null|referential action; the default `Restrict` is filled in; `null` on the non-owning end|
|`relationName`|string or null|`name:` of `@relation`, if given|
|`inverse`|string|name of the relation field on the target that forms the other end|
|`origin`|Origin|`own` or `mixin`|
|`attributes`|array of Attribute|user and plugin attributes|
|`span`|Span|position of the name|

- `references` is always filled in on the owning end, also when the schema left it to the default.
- The shape is already resolved: an owning end is always `one`; its other end is `one` and `nullable` for a one-to-one relation, `many` for a one-to-many relation.
- The foreign-key fields themselves are ordinary entries of `fields`.

Example, the owning end `user` of `Order`:

```json
{
  "name": {
    "source": "user",
    "words": [
      "user"
    ]
  },
  "doc": null,
  "target": "shop::users::User",
  "cardinality": "one",
  "nullable": false,
  "owning": true,
  "fields": [
    "userId"
  ],
  "references": [
    "id"
  ],
  "onDelete": "Cascade",
  "relationName": null,
  "inverse": "orders",
  "origin": {
    "kind": "own"
  },
  "attributes": [],
  "span": {
    "file": "shop/orders.linca",
    "line": 21,
    "column": 3
  }
}
```

---

## 8. What is not in the IR

- Source text, ordinary comments and imports.
- Attribute declarations and attribute sets.
- View operators and `with` clauses as written; only their results.
- The `@route` prefix of a service and the path of a method attribute as written; only the full path of each operation.
- The rules that place parameters and derive a binding; only the binding.
- Names in the convention of any target language.
- Warnings and errors.
- Plugin configuration, capabilities and anything else that belongs to one plugin run. These are part of the plugin protocol.

---

## 9. Canonical form

The compiler MUST write the IR in canonical form, so the same schema gives the same bytes.

- **Key order** is the order of `properties` in the JSON Schema, which is the order of the tables in this document.
- **Array order** is defined for every array in its table: source order unless stated otherwise; `modules` sorted by `id`; `declarations` by module and then by source order.
- **Strings** escape only `"`, `\` and control characters. Non-ASCII characters are written as they are.
- **Two layouts**, same content:
    - _compact_: no insignificant whitespace. This is what a plugin receives.
    - _pretty_: two-space indentation, one key per line, LF line endings, a final newline. This is what is shown to people and stored in the test data.

A reader MUST accept both layouts and MUST NOT depend on key order.

---

## 10. Versioning

- `irVersion` is `MAJOR.MINOR`.
- A **minor** version only adds: new keys, and new kinds of declaration. A reader written for an older minor version keeps working, because it ignores what it does not know ([[#2.4 Unknown content]]).
- A **major** version may remove or rename keys, change their meaning, or add a kind of type, value or origin.
- While the major version is `0`, any version may break. A plugin states the versions it supports through the plugin protocol.

---

## 11. Full example

The schema of Section 13 of [[Specification]] (three modules, eleven declarations) is given in full in `examples/shop/shop.ir.json`, next to its sources in `examples/shop/schema/`. The file validates against `schema/linca-ir.schema.json`.

Skeleton of that document:

```json
{
  "irVersion": "0.1",
  "languageVersion": "0.1",
  "modules": [
    { "id": "shop::common", "file": "shop/common.linca" },
    { "id": "shop::orders", "file": "shop/orders.linca" },
    { "id": "shop::users", "file": "shop/users.linca" }
  ],
  "declarations": [
    { "kind": "mixin",  "id": "shop::common::Timestamps" },
    { "kind": "type",   "id": "shop::common::UserId" },
    { "kind": "record", "id": "shop::orders::Address" },
    { "kind": "mixin",  "id": "shop::orders::PaymentBase" },
    { "kind": "record", "id": "shop::orders::CardPayment" },
    { "kind": "record", "id": "shop::orders::BankTransfer" },
    { "kind": "union",  "id": "shop::orders::Payment" },
    { "kind": "model",  "id": "shop::orders::Order" },
    { "kind": "enum",   "id": "shop::users::Role" },
    { "kind": "model",  "id": "shop::users::User" },
    { "kind": "view",   "id": "shop::users::PublicUser" }
  ]
}
```

(Each declaration is shortened to two keys here.)

Things to notice in the full file:

- `User.fields` starts with `createdAt` and `updatedAt`, with `origin.kind` of `mixin`; the mixin `Timestamps` is still listed as a declaration.
- The attribute sets `id` and `email` have left no trace: `User.id` has a `default`, `User.primaryKey` is `["id"]`, `User.email` has `constraints`.
- `PublicUser.fields` has no `passwordHash`; `id` and `createdAt` have lost their generated defaults, `role` has kept `Member`.
- `orders` of `User` and `user` of `Order` are in `relations`, not in `fields`; `userId` is an ordinary field of `Order`.

The example has no services. Error and service nodes are shown in [[#6.8 Error]] and [[#6.9 Service]], for the schema of Section 17.9 of [[Specification]].

---

## 12. Decisions and open questions

Decided:

- The IR is JSON; its normative definition is a JSON Schema.
- One document for the whole schema, whatever the number of source files ([[#3. Document]]).
- A flat list of declarations linked by id ([[#1. Overview]]).
- Arrays for everything ordered; every key always present; `null` for no value and `[]` for no elements ([[#2.3 `null` and empty]]).
- Core attributes are properties; only user and plugin attributes are Attribute nodes ([[#4.7 Attribute]]).
- Every name carries its words ([[#4.1 Name]]).
- No duplicated facts and no names in target conventions ([[#8. What is not in the IR]]).
- Compact and pretty layouts of one canonical form ([[#9. Canonical form]]).
- Services and errors are two more kinds of declaration; an error is referred to by id and is never the target of a TypeRef ([[#6.8 Error]], [[#6.9 Service]]).
- Every operation has a resolved HTTP binding, also when the schema has no HTTP attribute for it ([[#6.9 Service]]).

Proposed in this draft, to confirm:

- Numeric values from the schema are strings ([[#2.1 Encoding]]).
- The word-splitting rule ([[#4.1 Name]]).
- Spans on every named item ([[#4.2 Span]]).
- Field constraints are effective, including those of the domain type ([[#4.5 Constraints]]).
- Relation fields are listed in `relations`, apart from `fields` ([[#7. Relations]]).
- Readers ignore unknown keys and unknown declaration kinds, and fail on other unknown kinds ([[#2.4 Unknown content]]).
- Everything about HTTP is under keys named `http`, so the rest of an operation is transport-neutral ([[#6.9 Service]]).
- The path of an operation is an array of segments, not a string ([[#6.9 Service]]).
- The placement of a parameter is on the parameter (`http.in`), and the shape of the body is on the operation (`http.body`) ([[#6.9 Service]]).
- The `@route` prefix of a service is not kept ([[#8. What is not in the IR]]).

Open:

1. **Attribute sets and core attributes.** The language specification (7.4) says the IR records the set a use came from. Here that is true only for user and plugin attributes (`set`). A core attribute that came from a set becomes a property and the set is not recorded. Either the sentence in the language specification is narrowed, or a node gets a list of the sets applied to it.
2. **Relation fields in mixins.** The language does not say whether a mixin may contain a relation field. Here a Mixin declaration lists data fields only; a relation that came from a mixin appears in the models that use it, with `origin.kind` of `mixin`.
3. **Word splitting of mixed acronyms.** `OAuth2Token` gives `o`, `auth2`, `token`. A rule that handles such names needs a dictionary; an explicit override in the schema may be simpler.
4. **Size.** The full example is about 34 kB pretty and 17 kB compact for 11 declarations and 27 fields. Spans and always-present keys are most of it. To measure against real schemas before changing anything.
5. **IR on disk.** Whether the IR is also written next to the schema and committed, as Prisma 8 does with `contract.json`, or exists only between the compiler and a plugin. The format does not depend on the answer.
6. **Version.** `service` and `error` are new kinds of declaration, which is a minor change by [[#10. Versioning]]. This draft keeps `irVersion` at `0.1`, because no version has been released.
7. **Inputs and outputs.** Whether a declaration records that it is reachable from a parameter, from a result or from both. It depends on open question 2 of the language specification.
