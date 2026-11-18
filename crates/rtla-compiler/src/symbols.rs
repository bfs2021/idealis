use crate::analysis::{Node, analysis_children};
use crate::{Result, unsupported};
use rtla::ast::{DefinitionKind, Module};

pub(crate) fn validate(module: &Module) -> Result<()> {
    let conflict = |name: &str| {
        let builtin = matches!(
            name,
            "STRING"
                | "TRUE"
                | "FALSE"
                | "BOOLEAN"
                | "="
                | "#"
                | "'"
                | "~"
                | "/\\"
                | "\\/"
                | "<=>"
                | "=>"
                | "SUBSET"
                | "UNION"
                | "DOMAIN"
                | "\\subseteq"
                | "\\in"
                | "\\notin"
                | "\\"
                | "\\cap"
                | "\\cup"
                | "\\X"
                | "~>"
                | "[]"
                | "<>"
                | "ENABLED"
                | "UNCHANGED"
                | "\\cdot"
                | "-+->"
        );
        let imported = module.extends.iter().any(|module| match module.as_str() {
            "Naturals" | "Integers" => {
                matches!(
                    name,
                    "+" | "-" | "*" | "^" | "<" | ">" | "<=" | ">=" | "%" | "\\div" | ".."
                ) || module == "Integers" && name == "-."
            }
            "Sequences" => name == "\\o",
            "TLC" => matches!(name, ":>" | "@@"),
            _ => false,
        });
        if builtin || imported {
            unsupported(format!("builtin or imported symbol {name}"))
        } else {
            Ok(())
        }
    };
    for declaration in module.constants.iter().chain(&module.variables) {
        conflict(&declaration.name)?;
    }
    let mut pending: Vec<_> = module
        .definitions
        .iter()
        .map(Node::SourceDefinition)
        .chain(
            module
                .assumptions
                .iter()
                .map(|assumption| Node::Source(&assumption.expression)),
        )
        .chain(
            module
                .theorems
                .iter()
                .map(|theorem| Node::Source(&theorem.statement)),
        )
        .collect();
    while let Some(node) = pending.pop() {
        if let Node::SourceDefinition(definition) = node {
            conflict(&definition.name)?;
            if let DefinitionKind::Operator { params } = &definition.head {
                for parameter in params {
                    conflict(&parameter.name)?;
                }
            }
        }
        pending.extend(analysis_children(node).into_iter().map(|edge| edge.node));
    }
    Ok(())
}
