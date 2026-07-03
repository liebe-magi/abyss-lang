pub use crate::span::Span;

/// Expressions — nodes that evaluate to a value.
///
/// `Oracle` lives here (not in [`Stmt`]) because AbySS allows match
/// expressions in value position (`forge x: arcana = oracle(y) { … };`);
/// an oracle used as a statement flows through [`Stmt::Expr`].
#[derive(Debug, Clone)]
pub enum Expr {
    Omen(bool, Option<Span>),
    Arcana(i64, Option<Span>),
    Aether(f64, Option<Span>),
    Rune(String, Option<Span>),
    Abyss(Option<Span>),
    Add(Box<Expr>, Box<Expr>, Option<Span>),
    Sub(Box<Expr>, Box<Expr>, Option<Span>),
    Mul(Box<Expr>, Box<Expr>, Option<Span>),
    Div(Box<Expr>, Box<Expr>, Option<Span>),
    Mod(Box<Expr>, Box<Expr>, Option<Span>),
    PowArcana(Box<Expr>, Box<Expr>, Option<Span>),
    PowAether(Box<Expr>, Box<Expr>, Option<Span>),
    Equal(Box<Expr>, Box<Expr>, Option<Span>),
    NotEqual(Box<Expr>, Box<Expr>, Option<Span>),
    LessThan(Box<Expr>, Box<Expr>, Option<Span>),
    LessThanOrEqual(Box<Expr>, Box<Expr>, Option<Span>),
    GreaterThan(Box<Expr>, Box<Expr>, Option<Span>),
    GreaterThanOrEqual(Box<Expr>, Box<Expr>, Option<Span>),
    LogicalAnd(Box<Expr>, Box<Expr>, Option<Span>),
    LogicalOr(Box<Expr>, Box<Expr>, Option<Span>),
    LogicalNot(Box<Expr>, Option<Span>),
    Var(String, Option<Span>),
    FuncCall {
        name: String,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    IndexAccess {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Option<Span>,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
        span: Option<Span>,
    },
    ListLiteral {
        elements: Vec<Expr>,
        span: Option<Span>,
    },
    MapLiteral {
        entries: Vec<(String, Expr)>,
        span: Option<Span>,
    },
    ArtifactLiteral {
        type_name: String,
        fields: Vec<(String, Expr)>,
        span: Option<Span>,
    },
    Oracle {
        is_match: bool,
        conditionals: Vec<ConditionalAssignment>,
        branches: Vec<OracleBranch>,
        span: Option<Span>,
    },
}

/// Statements — nodes executed for their effect. A bare expression in
/// statement position is wrapped in [`Stmt::Expr`].
#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Option<Span>),
    VarAssign {
        name: String,
        value: Expr,
        var_type: Type,
        is_morph: bool,
        span: Option<Span>,
    },
    Assignment {
        name: String,
        value: Expr,
        op: AssignmentOp,
        span: Option<Span>,
    },
    IndexAssignment {
        target: Expr,
        index: Expr,
        value: Expr,
        span: Option<Span>,
    },
    FieldAssignment {
        target: Expr,
        field: String,
        value: Expr,
        span: Option<Span>,
    },
    Reveal(Expr, Option<Span>),
    Block(Vec<Stmt>, Option<Span>),
    /// Never produced by the parser (comments are scrubbed before lexing);
    /// retained so hand-built trees — formatter tests, generated code —
    /// can round-trip comment text through `format`.
    Comment(String, Option<Span>),
    Orbit {
        params: Vec<OrbitParam>,
        body: Box<Stmt>,
        span: Option<Span>,
    },
    Revolve(Option<String>, Option<Span>),
    Eject(Option<String>, Option<Span>),
    Engrave {
        name: String,
        params: Vec<EngraveParam>,
        return_type: Type,
        body: Box<Stmt>,
        method_target: Option<ArtifactMethodTarget>,
        span: Option<Span>,
    },
    ArtifactDef {
        name: String,
        fields: Vec<ArtifactField>,
        span: Option<Span>,
    },
}

/// Patterns — the shapes an `oracle` match arm can take. Only meaningful
/// inside [`OracleBranch::pattern`]; a bare [`Pattern::Expr`] is evaluated
/// and compared against the scrutinee (in match mode a lone
/// `Expr::Var` instead introduces a fresh binding).
#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_` — matches anything, binds nothing.
    DontCare(Option<Span>),
    /// Scroll-shape pattern destructuring a `scroll` scrutinee.
    Scroll {
        elements: Vec<Pattern>,
        span: Option<Span>,
    },
    /// Rest segment inside a scroll pattern: `..rest` (named) or `..`
    /// (anonymous, drops the tail). Only valid as the final element.
    Rest {
        name: Option<String>,
        span: Option<Span>,
    },
    /// Artifact-shape pattern: `TypeName { field: sub_pattern, … }`.
    /// Unlisted fields are not matched (non-exhaustive by default).
    Artifact {
        type_name: String,
        fields: Vec<(String, Pattern)>,
        span: Option<Span>,
    },
    /// Lexicon-shape pattern: `{ "key": sub_pattern, … }`. Unlisted keys
    /// are not matched.
    Lexicon {
        entries: Vec<(String, Pattern)>,
        span: Option<Span>,
    },
    /// Fallback: an expression evaluated and compared against the
    /// scrutinee. In match mode a bare `Expr::Var` binds instead.
    Expr(Expr),
}

/// One `oracle` match arm: `(pattern, …) ward guard => body`.
#[derive(Debug, Clone)]
pub struct OracleBranch {
    pub pattern: Vec<Pattern>,
    pub guard: Option<Expr>,
    pub body: Stmt,
    pub span: Option<Span>,
}

/// One `orbit` loop parameter: `name = start..end` (op is `..` or `..=`).
#[derive(Debug, Clone)]
pub struct OrbitParam {
    pub name: String,
    pub start: Expr,
    pub end: Expr,
    pub op: String,
    pub span: Option<Span>,
}

/// One `engrave` parameter: `name: type` with optional `morph`.
#[derive(Debug, Clone)]
pub struct EngraveParam {
    pub name: String,
    pub param_type: Type,
    pub is_morph: bool,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct ArtifactField {
    pub name: String,
    pub field_type: Type,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct ArtifactMethodTarget {
    pub artifact: String,
    pub requires_morph: bool,
}

/// Represents a conditional assignment within an oracle statement.
#[derive(Debug, Clone)]
pub struct ConditionalAssignment {
    pub variable: String,
    pub expression: Box<Expr>,
    pub span: Option<Span>,
}

/// Represents the type of a variable or expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Arcana,
    Aether,
    Rune,
    Omen,
    Abyss,
    Scroll,
    Lexicon,
    Materia,
    Glyph,
    Artifact(String),
}

/// Represents an assignment operation.
#[derive(Debug, Clone)]
pub enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowArcanaAssign,
    PowAetherAssign,
}
