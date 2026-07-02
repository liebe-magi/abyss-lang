pub use crate::span::Span;

/// Represents the abstract syntax tree (AST) for the language.
#[derive(Debug, Clone)]
pub enum AST {
    Statement(Box<AST>, Option<Span>),
    Omen(bool, Option<Span>),
    Arcana(i64, Option<Span>),
    Aether(f64, Option<Span>),
    Rune(String, Option<Span>),
    Abyss(Option<Span>),
    Add(Box<AST>, Box<AST>, Option<Span>),
    Sub(Box<AST>, Box<AST>, Option<Span>),
    Mul(Box<AST>, Box<AST>, Option<Span>),
    Div(Box<AST>, Box<AST>, Option<Span>),
    Mod(Box<AST>, Box<AST>, Option<Span>),
    PowArcana(Box<AST>, Box<AST>, Option<Span>),
    PowAether(Box<AST>, Box<AST>, Option<Span>),
    Equal(Box<AST>, Box<AST>, Option<Span>),
    NotEqual(Box<AST>, Box<AST>, Option<Span>),
    LessThan(Box<AST>, Box<AST>, Option<Span>),
    LessThanOrEqual(Box<AST>, Box<AST>, Option<Span>),
    GreaterThan(Box<AST>, Box<AST>, Option<Span>),
    GreaterThanOrEqual(Box<AST>, Box<AST>, Option<Span>),
    LogicalAnd(Box<AST>, Box<AST>, Option<Span>),
    LogicalOr(Box<AST>, Box<AST>, Option<Span>),
    LogicalNot(Box<AST>, Option<Span>),
    VarAssign {
        name: String,
        value: Box<AST>,
        var_type: Type,
        is_morph: bool,
        line_info: Option<Span>,
    },
    Assignment {
        name: String,
        value: Box<AST>,
        op: AssignmentOp,
        line_info: Option<Span>,
    },
    Var(String, Option<Span>),
    Reveal(Box<AST>, Option<Span>),
    Oracle {
        is_match: bool,
        conditionals: Vec<ConditionalAssignment>,
        branches: Vec<AST>,
        line_info: Option<Span>,
    },
    OracleBranch {
        pattern: Vec<AST>,
        guard: Option<Box<AST>>,
        body: Box<AST>,
        line_info: Option<Span>,
    },
    OracleDontCareItem(Option<Span>),
    /// Scroll-shape pattern that destructures a `scroll` scrutinee into
    /// its elements. Each element is one of: `OracleDontCareItem`,
    /// `OracleScrollRest`, `Var(name)` (binding), or any other AST node
    /// (treated as a literal expression to compare against).
    OracleScrollPattern {
        elements: Vec<AST>,
        line_info: Option<Span>,
    },
    /// Rest segment inside an `OracleScrollPattern`. `name = Some("rest")`
    /// for `..rest` (binds the unmatched tail to a fresh sub-scroll);
    /// `name = None` for `..` (anonymous, drops the tail).
    OracleScrollRest {
        name: Option<String>,
        line_info: Option<Span>,
    },
    /// Artifact-shape pattern that matches a `TypeName { field, … }`
    /// scrutinee. Each `(field_name, sub_pattern)` entry pulls the named
    /// field out of the artifact and matches it against `sub_pattern`
    /// (typically `Var` for binding, a literal for compare, or
    /// `OracleDontCareItem` to ignore). Fields not listed here are not
    /// matched against — the pattern is non-exhaustive by default, so
    /// users can pick out only the fields they care about.
    OracleArtifactPattern {
        type_name: String,
        fields: Vec<(String, AST)>,
        line_info: Option<Span>,
    },
    /// Lexicon-shape pattern that matches a `{ "key": value, … }`
    /// scrutinee. Each `(key, sub_pattern)` entry pulls the named entry
    /// out of the lexicon and matches it against `sub_pattern`. Keys not
    /// listed here are not matched against — the pattern is
    /// non-exhaustive by default, mirroring the artifact pattern's
    /// "pick what you need" ergonomics.
    OracleLexiconPattern {
        entries: Vec<(String, AST)>,
        line_info: Option<Span>,
    },
    Block(Vec<AST>, Option<Span>),
    Comment(String, Option<Span>),
    Orbit {
        params: Vec<AST>,
        body: Box<AST>,
        line_info: Option<Span>,
    },
    OrbitParam {
        name: String,
        start: Box<AST>,
        end: Box<AST>,
        op: String,
        line_info: Option<Span>,
    },
    Resume(Option<String>, Option<Span>),
    Eject(Option<String>, Option<Span>),
    Engrave {
        name: String,
        params: Vec<AST>,
        return_type: Type,
        body: Box<AST>,
        method_target: Option<ArtifactMethodTarget>,
        line_info: Option<Span>,
    },
    EngraveParam {
        name: String,
        param_type: Type,
        is_morph: bool,
        line_info: Option<Span>,
    },
    FuncCall {
        name: String,
        args: Vec<AST>,
        line_info: Option<Span>,
    },
    ListLiteral {
        elements: Vec<AST>,
        line_info: Option<Span>,
    },
    MapLiteral {
        entries: Vec<(String, AST)>,
        line_info: Option<Span>,
    },
    IndexAccess {
        target: Box<AST>,
        index: Box<AST>,
        line_info: Option<Span>,
    },
    IndexAssignment {
        target: Box<AST>,
        index: Box<AST>,
        value: Box<AST>,
        line_info: Option<Span>,
    },
    ArtifactDef {
        name: String,
        fields: Vec<ArtifactField>,
        line_info: Option<Span>,
    },
    ArtifactLiteral {
        type_name: String,
        fields: Vec<(String, AST)>,
        line_info: Option<Span>,
    },
    FieldAccess {
        target: Box<AST>,
        field: String,
        line_info: Option<Span>,
    },
    FieldAssignment {
        target: Box<AST>,
        field: String,
        value: Box<AST>,
        line_info: Option<Span>,
    },
    MethodCall {
        receiver: Box<AST>,
        method: String,
        args: Vec<AST>,
        line_info: Option<Span>,
    },
}

#[derive(Debug, Clone)]
pub struct ArtifactField {
    pub name: String,
    pub field_type: Type,
    pub line_info: Option<Span>,
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
    pub expression: Box<AST>,
    pub line_info: Option<Span>,
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
