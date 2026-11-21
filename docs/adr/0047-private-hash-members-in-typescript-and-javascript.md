# 0047 – `#private` members are private in TypeScript and JavaScript

Status: accepted

## Context

ADR 0022 mapped every class member without a keyword to `public` and listed `#private` fields as a known limit, because `#name` is no keyword. In the language it is stronger than `private`: the member cannot be reached from outside the class at all. A reader of the documentation who sees `#secret` as public gets a wrong picture of the API (issue #81, review of PR #79).

The pack vocabulary already has a rule by the spelling of a name (`visibility.name_rules`, used by the Python pack for the leading underscore), so this needs no new vocabulary.

## Decision

The packs `typescript`, `tsx` and `javascript` get one name rule: a name that starts with `#` is `private`. It applies to fields, methods and accessors with a `#` name and only when no keyword decides (a `#name` cannot have `private` anyway). All other rules of ADR 0022 stay.

The limit "`#private` fields get the default (public)" in ADR 0022, section 3, no longer holds. Parameter properties of constructors stay out of the symbol tree (see issue #81): the nesting of symbols follows the syntax tree, so such a field would become a child of the constructor, not of the class. That needs a change of the pack vocabulary of its own (done by ADR 0083: they are fields of the class now).

## Consequences

- The visibility of `#name` members changes from `public` to `private` in new scans; the golden files of the three packs are updated.
- No change of the data model, of the storage or of the other packs.
