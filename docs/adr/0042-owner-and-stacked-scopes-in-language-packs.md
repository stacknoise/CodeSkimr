# 0042 – Owners of declarations written outside their type, and stacked file-scoped namespaces

Status: proposed

## Context

Issue #65 (follow-up of the basic tier, ADR 0020) lists three limits of the basic packs:

- **Go:** `func (a A) Run()`, `func (b *B) Run()` and `func Run()` in one file have the same qualified name (`<file>.Run`), so `get_symbol` and the name resolution are ambiguous.
- **C++:** a method defined outside its class (`void Order::add() {}`) loses the class (`shop::add`); two classes with the same method collide.
- **PHP:** a second namespace without braces (`namespace App; … namespace Other;`) is nested in the first instead of being its sibling.

ADR 0039 solved the same problem for Rust `impl` blocks with a block that is no symbol. Here the declaration itself is the symbol and only its owner is written elsewhere, so a block cannot express it.

## Decision

1. **A new capture `@owner`** in `symbols.scm`: the node whose text is the simple name of the type that the declaration belongs to. It may appear together with `@symbol.<kind>` and `@name` in the same pattern. A pack without it is unchanged.
2. **Name and parent.** The qualified name of the declaration is `<scope>::<Owner>::<name>` (with the separator of the pack). If a type (or type alias) of that name is declared in the same scope, or exactly one in the whole file before the declaration, it becomes the parent and its own qualified name is the prefix; otherwise the parent stays the enclosing symbol and only the name carries the owner. The mechanism is the one of ADR 0039 (a frame without a symbol), so containers and owners behave the same.
3. **Go:** the type of the receiver is the owner (`A` in `func (a *A) Run()`, also generic receivers). Go names start with the file (`file_in_names`), so a method in another file than its type is named with its own file (`b.A.Run`) and has no parent; the receiver type in the name already separates it from `Run` of another type.
4. **C++:** the name before `::` is the owner, also a class template (`Box<T>::get`). A longer path (`ns::Order::add`) and definitions outside the namespace of the class are not read as owners; they stay as before, with only the last name.
5. **A stretched scope ends at the next one.** A declaration with `@scope_end` (a namespace without braces holds the rest of the file) ends before the next declaration of the same kind under the same parent: `namespace A; … namespace B; …` gives two siblings. The span and the range of the first namespace end with the last node before the second one.
6. **Naming revision.** Names of Go and C++ methods change, so the revision of the naming rules (ADR 0036, ADR 0039) goes up, and the next scan of every project is a full scan.
7. **Golden tests** of Go, C++ and PHP show the cases (same method name on two types, pointer and generic receivers, a class template, two stacked namespaces).

## Consequences

- Methods of one type are children of it where the type is declared in the file, so metrics roll up to the type and the resolver finds the methods through their owner.
- **Limits.** A Go method whose type lies in another file has no link to it in the symbol tree. A C++ definition with a longer scope path stays without an owner. Two types of one name in one scope: the declaration belongs to the first one (same limit as ADR 0039). PHP namespaces inside braces are unchanged.
