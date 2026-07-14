# mugens-domain

`mugens-domain` is the place for Mugens-specific domain types.

It should contain data structures and errors that describe the application
domain independently from Unode rendering and plugin runtime details.

## Owns

- Mugens domain models;
- shared domain error types;
- DTOs that need to be shared across app, bridge, and plugins.

## Does Not Own

- Unode AST or renderer primitives;
- host-call registration;
- permission enforcement;
- app shell or terminal/web rendering.

Keep this crate free of UI runtime concerns so it can be reused by bridge and
application code.
