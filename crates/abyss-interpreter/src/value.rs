//! The interpreter's core value representation.
//!
//! [`Value`] is what variables hold at runtime. Collections (`Scroll`,
//! `Lexicon`) and artifact instances are reference-counted with interior
//! mutability so assignment aliases rather than deep-copies; deep copies
//! are made explicitly where the language semantics require them (see
//! `eval::values`). Re-exported from [`crate::env`] for backwards
//! compatibility with the pre-split module layout.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use abyss_core::ast::Type;

use crate::artifact::ArtifactHandle;

/// Represents the value stored in a variable, including primitive scalars, collections,
/// glyphs (type handles), and artifact instances.
#[derive(Debug, Clone)]
pub enum Value {
    Omen(bool),
    Arcana(i64),
    Aether(f64),
    Rune(Rc<String>),
    Abyss,
    Scroll(Rc<RefCell<Vec<Value>>>),
    Lexicon(Rc<RefCell<HashMap<String, Value>>>),
    Glyph(Type),
    Artifact(ArtifactHandle),
}
