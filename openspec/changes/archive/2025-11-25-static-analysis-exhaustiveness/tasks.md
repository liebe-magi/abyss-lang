# Tasks

- [x] **Schema Migration** <!-- id: 0 -->
    - [x] Move `ArtifactSchema` and `SpectrumSchema` to `abyss-core/src/types.rs`. <!-- id: 1 -->
    - [x] Update `abyss-interpreter` to use schemas from `abyss-core`. <!-- id: 2 -->
- [x] **SymbolTable Expansion** <!-- id: 3 -->
    - [x] Extend `SymbolTable` in `abyss-core/src/semantic.rs` to support `Spectrum` and `Artifact` definitions. <!-- id: 4 -->
- [x] **Analyzer Scaffold** <!-- id: 5 -->
    - [x] Create `abyss-core/src/analysis.rs`. <!-- id: 6 -->
    - [x] Implement `Analyzer` struct and basic AST traversal. <!-- id: 7 -->
- [x] **Exhaustiveness Logic** <!-- id: 8 -->
    - [x] Implement `check_exhaustiveness` in `Analyzer`. <!-- id: 9 -->
    - [x] Implement primitive type checks (catch-all requirement). <!-- id: 10 -->
    - [x] Implement Spectrum type checks (variant coverage). <!-- id: 11 -->
- [x] **Integration** <!-- id: 12 -->
    - [x] Integrate `Analyzer` into the execution pipeline (CLI/Interpreter). <!-- id: 13 -->
    - [x] Remove redundant runtime checks in `abyss-interpreter` (optional/cleanup). <!-- id: 14 -->
