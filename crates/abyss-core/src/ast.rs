/// Represents line and column information for debugging purposes.
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub line: usize,
    pub column: usize,
}

impl LineInfo {
    /// Creates a `LineInfo` from explicit 1-based line and column values.
    pub fn new(line: usize, column: usize) -> Self {
        LineInfo { line, column }
    }
}

/// Represents the abstract syntax tree (AST) for the language.
#[derive(Debug, Clone)]
pub enum AST {
    Statement(Box<AST>, Option<LineInfo>),
    Omen(bool, Option<LineInfo>),
    Arcana(i64, Option<LineInfo>),
    Aether(f64, Option<LineInfo>),
    Rune(String, Option<LineInfo>),
    Abyss(Option<LineInfo>),
    Add(Box<AST>, Box<AST>, Option<LineInfo>),
    Sub(Box<AST>, Box<AST>, Option<LineInfo>),
    Mul(Box<AST>, Box<AST>, Option<LineInfo>),
    Div(Box<AST>, Box<AST>, Option<LineInfo>),
    Mod(Box<AST>, Box<AST>, Option<LineInfo>),
    PowArcana(Box<AST>, Box<AST>, Option<LineInfo>),
    PowAether(Box<AST>, Box<AST>, Option<LineInfo>),
    Equal(Box<AST>, Box<AST>, Option<LineInfo>),
    NotEqual(Box<AST>, Box<AST>, Option<LineInfo>),
    LessThan(Box<AST>, Box<AST>, Option<LineInfo>),
    LessThanOrEqual(Box<AST>, Box<AST>, Option<LineInfo>),
    GreaterThan(Box<AST>, Box<AST>, Option<LineInfo>),
    GreaterThanOrEqual(Box<AST>, Box<AST>, Option<LineInfo>),
    LogicalAnd(Box<AST>, Box<AST>, Option<LineInfo>),
    LogicalOr(Box<AST>, Box<AST>, Option<LineInfo>),
    LogicalNot(Box<AST>, Option<LineInfo>),
    VarAssign {
        name: String,
        value: Box<AST>,
        var_type: Type,
        is_morph: bool,
        line_info: Option<LineInfo>,
    },
    Assignment {
        name: String,
        value: Box<AST>,
        op: AssignmentOp,
        line_info: Option<LineInfo>,
    },
    Var(String, Option<LineInfo>),
    Reveal(Box<AST>, Option<LineInfo>),
    Oracle {
        is_match: bool,
        conditionals: Vec<ConditionalAssignment>,
        branches: Vec<AST>,
        line_info: Option<LineInfo>,
    },
    OracleBranch {
        pattern: Vec<AST>,
        guard: Option<Box<AST>>,
        body: Box<AST>,
        line_info: Option<LineInfo>,
    },
    OracleDontCareItem(Option<LineInfo>),
    /// Scroll-shape pattern that destructures a `scroll` scrutinee into
    /// its elements. Each element is one of: `OracleDontCareItem`,
    /// `OracleScrollRest`, `Var(name)` (binding), or any other AST node
    /// (treated as a literal expression to compare against).
    OracleScrollPattern {
        elements: Vec<AST>,
        line_info: Option<LineInfo>,
    },
    /// Rest segment inside an `OracleScrollPattern`. `name = Some("rest")`
    /// for `..rest` (binds the unmatched tail to a fresh sub-scroll);
    /// `name = None` for `..` (anonymous, drops the tail).
    OracleScrollRest {
        name: Option<String>,
        line_info: Option<LineInfo>,
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
        line_info: Option<LineInfo>,
    },
    /// Lexicon-shape pattern that matches a `{ "key": value, … }`
    /// scrutinee. Each `(key, sub_pattern)` entry pulls the named entry
    /// out of the lexicon and matches it against `sub_pattern`. Keys not
    /// listed here are not matched against — the pattern is
    /// non-exhaustive by default, mirroring the artifact pattern's
    /// "pick what you need" ergonomics.
    OracleLexiconPattern {
        entries: Vec<(String, AST)>,
        line_info: Option<LineInfo>,
    },
    Block(Vec<AST>, Option<LineInfo>),
    Comment(String, Option<LineInfo>),
    Orbit {
        params: Vec<AST>,
        body: Box<AST>,
        line_info: Option<LineInfo>,
    },
    OrbitParam {
        name: String,
        start: Box<AST>,
        end: Box<AST>,
        op: String,
        line_info: Option<LineInfo>,
    },
    Resume(Option<String>, Option<LineInfo>),
    Eject(Option<String>, Option<LineInfo>),
    Engrave {
        name: String,
        params: Vec<AST>,
        return_type: Type,
        body: Box<AST>,
        method_target: Option<ArtifactMethodTarget>,
        line_info: Option<LineInfo>,
    },
    EngraveParam {
        name: String,
        param_type: Type,
        is_morph: bool,
        line_info: Option<LineInfo>,
    },
    FuncCall {
        name: String,
        args: Vec<AST>,
        line_info: Option<LineInfo>,
    },
    ListLiteral {
        elements: Vec<AST>,
        line_info: Option<LineInfo>,
    },
    MapLiteral {
        entries: Vec<(String, AST)>,
        line_info: Option<LineInfo>,
    },
    IndexAccess {
        target: Box<AST>,
        index: Box<AST>,
        line_info: Option<LineInfo>,
    },
    IndexAssignment {
        target: Box<AST>,
        index: Box<AST>,
        value: Box<AST>,
        line_info: Option<LineInfo>,
    },
    ArtifactDef {
        name: String,
        fields: Vec<ArtifactField>,
        line_info: Option<LineInfo>,
    },
    ArtifactLiteral {
        type_name: String,
        fields: Vec<(String, AST)>,
        line_info: Option<LineInfo>,
    },
    FieldAccess {
        target: Box<AST>,
        field: String,
        line_info: Option<LineInfo>,
    },
    FieldAssignment {
        target: Box<AST>,
        field: String,
        value: Box<AST>,
        line_info: Option<LineInfo>,
    },
    MethodCall {
        receiver: Box<AST>,
        method: String,
        args: Vec<AST>,
        line_info: Option<LineInfo>,
    },
}

#[derive(Debug, Clone)]
pub struct ArtifactField {
    pub name: String,
    pub field_type: Type,
    pub line_info: Option<LineInfo>,
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
    pub line_info: Option<LineInfo>,
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
