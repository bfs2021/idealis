use crate::{CoreError, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Expression {
    Prime {
        argument: Box<Self>,
    },
    Data {
        value: Value,
    },
    Bound {
        slot: usize,
    },
    Old,
    UnmatchedCase {
        location: String,
    },
    Set {
        values: Vec<Self>,
    },
    Tuple {
        values: Vec<Self>,
    },
    Record {
        fields: Vec<(String, Self)>,
    },
    RecordSet {
        fields: Vec<(String, Self)>,
    },
    At {
        target: Box<Self>,
        key: Box<Self>,
    },
    Update {
        target: Box<Self>,
        updates: Vec<Update>,
    },
    Forall {
        bounds: Vec<Binding>,
        body: Box<Self>,
    },
    Exists {
        bounds: Vec<Binding>,
        body: Box<Self>,
    },
    Choose {
        binding: Binding,
        body: Box<Self>,
    },
    Function {
        bounds: Vec<Binding>,
        body: Box<Self>,
    },
    Filter {
        bounds: Vec<Binding>,
        body: Box<Self>,
    },
    MapSet {
        bounds: Vec<Binding>,
        body: Box<Self>,
    },
    Fold {
        accumulator: usize,
        initial: Box<Self>,
        collection: Box<Self>,
        order: FoldOrder,
        body: Box<Self>,
    },
    SelectSeq {
        slot: usize,
        sequence: Box<Self>,
        body: Box<Self>,
    },
    SortSeq {
        slot: usize,
        sequence: Box<Self>,
        body: Box<Self>,
    },
    Assert {
        condition: Box<Self>,
        message: Box<Self>,
    },
    Domain {
        argument: Box<Self>,
    },
    VariantTag {
        argument: Box<Self>,
    },
    Cardinality {
        argument: Box<Self>,
    },
    SequenceSet {
        argument: Box<Self>,
    },
    Len {
        argument: Box<Self>,
    },
    Head {
        argument: Box<Self>,
    },
    Tail {
        argument: Box<Self>,
    },
    Powerset {
        argument: Box<Self>,
    },
    Union {
        argument: Box<Self>,
    },
    Cup {
        left: Box<Self>,
        right: Box<Self>,
    },
    Cap {
        left: Box<Self>,
        right: Box<Self>,
    },
    Difference {
        left: Box<Self>,
        right: Box<Self>,
    },
    SubsetEq {
        left: Box<Self>,
        right: Box<Self>,
    },
    Append {
        left: Box<Self>,
        right: Box<Self>,
    },
    Concat {
        left: Box<Self>,
        right: Box<Self>,
    },
    FunctionSet {
        left: Box<Self>,
        right: Box<Self>,
    },
    FunctionIn {
        left: Box<Self>,
        right: Box<Self>,
    },
    SequenceIn {
        left: Box<Self>,
        right: Box<Self>,
    },
    Subseq {
        sequence: Box<Self>,
        from: Box<Self>,
        to: Box<Self>,
    },
    Enabled {
        branches: Vec<Branch>,
        #[serde(default)]
        subscript: Option<Box<Self>>,
    },
    Literal {
        value: Scalar,
    },
    Read {
        slot: usize,
        next: bool,
    },
    Not {
        argument: Box<Self>,
    },
    Neg {
        argument: Box<Self>,
    },
    Add {
        left: Box<Self>,
        right: Box<Self>,
    },
    Sub {
        left: Box<Self>,
        right: Box<Self>,
    },
    Mul {
        left: Box<Self>,
        right: Box<Self>,
    },
    Div {
        left: Box<Self>,
        right: Box<Self>,
    },
    Mod {
        left: Box<Self>,
        right: Box<Self>,
    },
    Pow {
        left: Box<Self>,
        right: Box<Self>,
    },
    Eq {
        left: Box<Self>,
        right: Box<Self>,
    },
    Ne {
        left: Box<Self>,
        right: Box<Self>,
    },
    Lt {
        left: Box<Self>,
        right: Box<Self>,
    },
    Le {
        left: Box<Self>,
        right: Box<Self>,
    },
    Gt {
        left: Box<Self>,
        right: Box<Self>,
    },
    Ge {
        left: Box<Self>,
        right: Box<Self>,
    },
    And {
        left: Box<Self>,
        right: Box<Self>,
    },
    Or {
        left: Box<Self>,
        right: Box<Self>,
    },
    Implies {
        left: Box<Self>,
        right: Box<Self>,
    },
    If {
        condition: Box<Self>,
        consequent: Box<Self>,
        alternative: Box<Self>,
    },
    In {
        value: Box<Self>,
        domain: Domain,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum FoldOrder {
    SetMembers,
    SequenceLeft,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Domain {
    Value {
        value: Box<Expression>,
    },
    Range {
        lower: Box<Expression>,
        upper: Box<Expression>,
    },
    Set {
        values: Vec<Expression>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Step {
    Bind {
        slot: usize,
        domain: Domain,
        #[serde(rename = "tupleSize", default, skip_serializing_if = "Option::is_none")]
        tuple_size: Option<usize>,
    },
    Guard {
        expression: Expression,
    },
    Assign {
        slot: usize,
        value: Expression,
    },
    Choose {
        slot: usize,
        domain: Domain,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(rename = "tupleSize", default, skip_serializing_if = "Option::is_none")]
    pub tuple_size: Option<usize>,
    pub slot: usize,
    pub domain: Domain,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub path: Vec<Expression>,
    pub value: Expression,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub name: String,
    pub expression: Expression,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SafetyProgram {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub variables: Vec<String>,
    pub initial: Vec<Branch>,
    pub next: Vec<Branch>,
    pub invariants: Vec<Predicate>,
    pub assumptions: Vec<Predicate>,
    pub check_deadlock: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_properties: Vec<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_properties: Vec<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<TemporalProperty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fairness: Vec<TemporalFormula>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_constraints: Vec<Predicate>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SafetyBounds {
    pub max_depth: u32,
    pub max_states: u32,
    pub max_work: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_temporal_nodes: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafetyTraceStep {
    pub action: Option<String>,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafetyViolation {
    pub name: String,
    pub trace: Vec<SafetyTraceStep>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "loopStart")]
    pub loop_start: Option<usize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "loopAction"
    )]
    pub loop_action: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafetyResult {
    pub status: String,
    pub reason: Option<String>,
    pub states: usize,
    pub generated: u32,
    pub depth: u32,
    pub work: u32,
    pub violation: Option<SafetyViolation>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SafetyActionCoverage {
    pub name: String,
    pub attempts: u32,
    pub enabled: u32,
    pub generated: u32,
    pub distinct: usize,
    pub work: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SafetyCoverage {
    pub initial: Vec<SafetyActionCoverage>,
    pub next: Vec<SafetyActionCoverage>,
    pub unattributed_work: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafetyCoverageReport {
    pub result: SafetyResult,
    pub coverage: SafetyCoverage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "Value")]
pub enum Scalar {
    Bool(bool),
    Int(i32),
    Ref(u32),
}

impl TryFrom<Value> for Scalar {
    type Error = CoreError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl Serialize for Scalar {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            Self::Bool(value) => Value::Bool(*value),
            Self::Int(value) => Value::Int(value.to_string()),
            Self::Ref(_) => return Err(serde::ser::Error::custom("Internal value reference")),
        };
        value.serialize(serializer)
    }
}

impl Scalar {
    pub(crate) fn parse(value: &Value) -> Result<Self, CoreError> {
        match value {
            Value::Bool(value) => Ok(Self::Bool(*value)),
            Value::Int(value) => {
                let parsed = value.parse::<i32>().map_err(|_| {
                    CoreError::protocol("Safety integer exceeds signed 32 bit range")
                })?;
                if parsed.to_string() != *value {
                    return Err(CoreError::protocol("Noncanonical safety integer"));
                }
                Ok(Self::Int(parsed))
            }
            _ => Err(CoreError::protocol(
                "Safety values must be integers or booleans",
            )),
        }
    }
    pub(crate) fn integer(self) -> Result<i32, CoreError> {
        match self {
            Self::Int(value) => Ok(value),
            _ => Err(CoreError::protocol("Expected an integer")),
        }
    }
    pub(crate) fn boolean(self) -> Result<bool, CoreError> {
        match self {
            Self::Bool(value) => Ok(value),
            _ => Err(CoreError::protocol("Expected a boolean")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TemporalFormula {
    Atom {
        expression: Expression,
    },
    Not {
        formula: Box<Self>,
    },
    Always {
        formula: Box<Self>,
    },
    Eventually {
        formula: Box<Self>,
    },
    And {
        left: Box<Self>,
        right: Box<Self>,
    },
    Or {
        left: Box<Self>,
        right: Box<Self>,
    },
    Forall {
        bounds: Vec<Binding>,
        formula: Box<Self>,
    },
    Exists {
        bounds: Vec<Binding>,
        formula: Box<Self>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalProperty {
    pub name: String,
    pub formula: TemporalFormula,
}
