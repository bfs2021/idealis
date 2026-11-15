use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    #[serde(flatten)]
    pub node: ExpressionKind,
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ExpressionKind {
    Number {
        value: u64,
    },
    Decimal {
        text: String,
    },
    String {
        value: String,
    },
    Identifier {
        name: String,
    },
    Apply {
        operator: String,
        args: Vec<Expression>,
    },
    And {
        operands: Vec<Expression>,
        #[serde(
            rename = "isBulleted",
            default,
            skip_serializing_if = "std::ops::Not::not"
        )]
        is_bulleted: bool,
    },
    Or {
        operands: Vec<Expression>,
        #[serde(
            rename = "isBulleted",
            default,
            skip_serializing_if = "std::ops::Not::not"
        )]
        is_bulleted: bool,
    },
    Prime {
        expression: Box<Expression>,
    },
    Unchanged {
        expression: Box<Expression>,
    },
    Enabled {
        expression: Box<Expression>,
    },
    Always {
        expression: Box<Expression>,
    },
    Eventually {
        expression: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        consequent: Box<Expression>,
        alternative: Box<Expression>,
    },
    Case {
        arms: Vec<CaseArm>,
        #[serde(skip_serializing_if = "Option::is_none")]
        other: Option<Box<Expression>>,
    },
    Let {
        definitions: Vec<Definition>,
        body: Box<Expression>,
    },
    Forall {
        bounds: Vec<Bound>,
        body: Box<Expression>,
    },
    Exists {
        bounds: Vec<Bound>,
        body: Box<Expression>,
    },
    UnboundedQuantifier {
        quantifier: String,
        names: Vec<String>,
        #[serde(rename = "isTemporal")]
        is_temporal: bool,
        body: Box<Expression>,
    },
    Choose {
        pattern: Pattern,
        #[serde(skip_serializing_if = "Option::is_none")]
        set: Option<Box<Expression>>,
        body: Box<Expression>,
    },
    SetEnumeration {
        elements: Vec<Expression>,
    },
    SetFilter {
        bound: Box<Bound>,
        predicate: Box<Expression>,
    },
    SetMap {
        expression: Box<Expression>,
        bounds: Vec<Bound>,
    },
    Tuple {
        elements: Vec<Expression>,
    },
    FunctionConstruction {
        bounds: Vec<Bound>,
        body: Box<Expression>,
    },
    FunctionSet {
        domain: Box<Expression>,
        range: Box<Expression>,
    },
    Record {
        fields: Vec<RecordField>,
    },
    RecordSet {
        fields: Vec<RecordField>,
    },
    Except {
        function: Box<Expression>,
        updates: Vec<ExceptUpdate>,
    },
    ExceptAt,
    FunctionApplication {
        function: Box<Expression>,
        args: Vec<Expression>,
    },
    Bracket {
        args: Vec<Expression>,
    },
    FieldAccess {
        record: Box<Expression>,
        field: String,
    },
    Product {
        factors: Vec<Expression>,
    },
    BoxAction {
        action: Box<Expression>,
        subscript: Box<Expression>,
    },
    AngleAction {
        action: Box<Expression>,
        subscript: Box<Expression>,
    },
    WeakFairness {
        subscript: Box<Expression>,
        action: Box<Expression>,
    },
    StrongFairness {
        subscript: Box<Expression>,
        action: Box<Expression>,
    },
    Label {
        name: String,
        params: Vec<String>,
        body: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bound {
    pub pattern: Pattern,
    pub set: Expression,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Pattern {
    Name { name: String },
    Tuple { names: Vec<String> },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseArm {
    pub condition: Expression,
    pub body: Expression,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    pub name: String,
    pub value: Expression,
    pub location: Location,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExceptUpdate {
    pub path: Vec<ExceptPath>,
    pub value: Expression,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ExceptPath {
    Field { name: String },
    Index { args: Vec<Expression> },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub name: String,
    pub arity: usize,
    pub location: Location,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    #[serde(flatten)]
    pub head: DefinitionKind,
    pub body: Expression,
    #[serde(rename = "isLocal")]
    pub is_local: bool,
    #[serde(rename = "isRecursive")]
    pub is_recursive: bool,
    pub location: Location,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DefinitionKind {
    Operator { params: Vec<Declaration> },
    Function { bounds: Vec<Bound> },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assumption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub expression: Expression,
    #[serde(rename = "isAxiom")]
    pub is_axiom: bool,
    pub location: Location,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theorem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub statement: Expression,
    pub location: Location,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub extends: Vec<String>,
    pub constants: Vec<Declaration>,
    pub variables: Vec<Declaration>,
    pub definitions: Vec<Definition>,
    pub assumptions: Vec<Assumption>,
    pub theorems: Vec<Theorem>,
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConfigValue {
    Number { value: i64 },
    String { value: String },
    Boolean { value: bool },
    ModelValue { name: String },
    Set { elements: Vec<Self> },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstantAssignment {
    pub name: String,
    pub value: ConfigValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<ConfigValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Substitution {
    pub name: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specification: Option<String>,
    pub invariants: Vec<String>,
    pub properties: Vec<String>,
    pub constraints: Vec<String>,
    pub action_constraints: Vec<String>,
    pub constants: Vec<ConstantAssignment>,
    pub substitutions: Vec<Substitution>,
    pub raw_constants: Vec<String>,
    pub check_deadlock: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symmetry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub postconditions: Vec<String>,
    pub possible: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periodic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rl_reward: Option<String>,
}
