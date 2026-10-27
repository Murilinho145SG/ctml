# CTML Basic Syntax

This document defines the initial syntax supported by the CTML lexer and parser.

## Document structure

A complete interface document uses a `headers` section followed by a `body` section:

```ctml
<headers>
    <application name="my-application" desktop />
    <stylesheet href="main.css" />
</headers>

<body>
    <div class="message">Hello, world!</div>
</body>
```

The parser requires one `headers` section followed by one `body` section, validates matching tags, and rejects duplicate attributes. An optional `stylesheet` element links a CSS file relative to the Rust application's `src/` directory. The current native CSS support is described in [css.md](css.md).

## Elements

Elements use opening and closing tags:

```ctml
<div class="panel">
    Content
</div>
```

An element with no children can use a self-closing tag:

```ctml
<application name="my-application" desktop />
```

Tag names are case-sensitive. Names start with an ASCII letter or underscore and may continue with ASCII letters, digits, underscores, and hyphens.

## Attributes

Attributes appear inside opening tags and are separated by whitespace. Values use double quotes. An attribute without `=` is a boolean attribute:

```ctml
<application name="my-application" desktop />
```

The initial lexer supports `\"` and `\\` escapes inside quoted values. Unquoted values and single-quoted strings are not part of the initial syntax.

## Text

Text between tags is preserved, including spaces and line breaks. The next `<` begins a tag; there is no text escape syntax in this initial version.

## Initial grammar

```text
document          = headers-section body-section
headers-section   = "<headers>" node* "</headers>"
body-section      = "<body>" node* "</body>"
node              = element | text
element           = paired-element | self-closing-element
paired-element    = start-tag node* end-tag
self-closing-element = "<" name attribute* "/>"
start-tag         = "<" name attribute* ">"
end-tag           = "</" name ">"
attribute         = name [ "=" quoted-string ]
quoted-string     = '"' character* '"'
```

The lexer emits tokens with source positions. The parser builds a tree and validates the document sections and paired tag names. The native demo compiles body nodes and linked CSS into Rust data during the build.
