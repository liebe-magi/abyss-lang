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
    Trans(Box<AST>, Type, Option<LineInfo>),
    Reveal(Box<AST>, Option<LineInfo>),
    Oracle {
        is_match: bool,
        conditionals: Vec<ConditionalAssignment>,
        branches: Vec<AST>,
        line_info: Option<LineInfo>,
    },
    OracleBranch {
        pattern: Vec<AST>,
        body: Box<AST>,
        line_info: Option<LineInfo>,
    },
    OracleDontCareItem(Option<LineInfo>),
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
        line_info: Option<LineInfo>,
    },
    EngraveParam {
        name: String,
        param_type: Type,
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
