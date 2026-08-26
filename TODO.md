# What's next?

 - [x] Param styles (renamed from `ParamType` in 0.4.0)
 - [ ] Remove vendor's databases from system types
 - [ ] More tests
 - [ ] More examples
 - [ ] Docs 
 - [ ] OS support
    - [x] Linux
    - [x] Windows
    - [ ] Macos

- [x] Add support for subtasks from string without file and paths
- [ ] Add suppport for chain parameters applying
- [x] SQL parameter binding — `prepare()` -> `(query, params)`
- [x] Base guards against SQL injection
- [x] Dialect-aware statement analysis (`analyze()` / `forbid=[...]`)
- [ ] Widen dialect coverage where `sqlparser` fails on vendor syntax
      (DuckDB `CREATE PERSISTENT SECRET`, `ATTACH ... (TYPE POSTGRES)`)
- [ ] Dialect-aware identifier quoting (`"x"` vs `` `x` ``)
- [ ] `executemany`-style binding for a list of parameter sets
- [ ] Async/batch execution helpers