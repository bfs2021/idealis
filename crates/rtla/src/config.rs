use crate::ast::{Config, ConfigValue, ConstantAssignment, Substitution};
use crate::lexer::{Kind, tokenize};
use crate::parser::Parser;
use crate::{ErrorKind, ParseError};

const KEYWORDS: &[&str] = &[
    "INIT",
    "NEXT",
    "SPECIFICATION",
    "CONSTANT",
    "CONSTANTS",
    "INVARIANT",
    "INVARIANTS",
    "PROPERTY",
    "PROPERTIES",
    "CONSTRAINT",
    "CONSTRAINTS",
    "ACTION_CONSTRAINT",
    "ACTION_CONSTRAINTS",
    "CHECK_DEADLOCK",
    "SYMMETRY",
    "VIEW",
    "ALIAS",
    "POSTCONDITION",
    "POSTCONDITIONS",
    "_POSSIBLE",
    "_PERIODIC",
    "_RL_REWARD",
];
fn name(parser: &mut Parser) -> Result<String, ParseError> {
    if KEYWORDS.contains(&parser.raw().text.as_str()) {
        return Err(parser.error("Expected a configuration argument"));
    }
    let mut name = parser.name()?.text;
    while parser.accept("!") {
        name.push('!');
        name.push_str(&parser.name()?.text);
    }
    Ok(name)
}
fn value(parser: &mut Parser, depth: usize) -> Result<ConfigValue, ParseError> {
    if depth >= crate::MAX_NESTING {
        return Err(ParseError {
            kind: ErrorKind::Unsupported,
            message: "Configuration value nesting exceeds 64".into(),
            location: parser.raw().location,
        });
    }
    let token = parser.advance()?;
    if token.kind == Kind::String {
        if token.raw.contains('\\') {
            return Err(parser.error("TLC configuration string escapes are not supported"));
        }
        return Ok(ConfigValue::String { value: token.text });
    }
    if token.text == "{" {
        let mut elements = vec![];
        if !parser.at("}") {
            elements.push(value(parser, depth + 1)?);
            while parser.accept(",") {
                elements.push(value(parser, depth + 1)?);
            }
        }
        parser.expect("}")?;
        return Ok(ConfigValue::Set { elements });
    }
    if token.kind == Kind::Number || token.text == "-" {
        let text = if token.text == "-" {
            let next = parser.advance()?;
            if next.kind != Kind::Number {
                return Err(parser.error("Expected integer after minus"));
            }
            format!("-{}", next.text)
        } else {
            token.text
        };
        let value = text
            .parse::<i64>()
            .ok()
            .filter(|value| value.unsigned_abs() <= 9_007_199_254_740_991)
            .ok_or_else(|| parser.error("Expected an exact decimal configuration integer"))?;
        return Ok(ConfigValue::Number { value });
    }
    if token.text == "TRUE" || token.text == "FALSE" {
        return Ok(ConfigValue::Boolean {
            value: token.text == "TRUE",
        });
    }
    if token.kind == Kind::Name && !KEYWORDS.contains(&token.text.as_str()) {
        return Ok(ConfigValue::ModelValue { name: token.text });
    }
    Err(parser.error("Expected configuration value"))
}
fn scope(parser: &mut Parser) -> Result<Option<String>, ParseError> {
    if !parser.accept("[") {
        return Ok(None);
    }
    let module = name(parser)?;
    parser.expect("]")?;
    Ok(Some(module))
}
pub fn parse_config(source: &str) -> Result<Config, ParseError> {
    let mut parser = Parser::new(tokenize(source, 0, false)?);
    let mut config = Config {
        init: None,
        next: None,
        specification: None,
        invariants: vec![],
        properties: vec![],
        constraints: vec![],
        action_constraints: vec![],
        constants: vec![],
        substitutions: vec![],
        raw_constants: vec![],
        check_deadlock: true,
        symmetry: None,
        view: None,
        alias: None,
        postconditions: vec![],
        possible: vec![],
        periodic: None,
        rl_reward: None,
    };
    let mut deadlock_set = false;
    while parser.raw().kind != Kind::End {
        let keyword = parser.advance()?;
        let single = match keyword.text.as_str() {
            "INIT" => Some(&mut config.init),
            "NEXT" => Some(&mut config.next),
            "SPECIFICATION" => Some(&mut config.specification),
            "SYMMETRY" => Some(&mut config.symmetry),
            "VIEW" => Some(&mut config.view),
            "ALIAS" => Some(&mut config.alias),
            "_PERIODIC" => Some(&mut config.periodic),
            "_RL_REWARD" => Some(&mut config.rl_reward),
            _ => None,
        };
        if let Some(entry) = single {
            if entry.is_some() {
                return Err(parser.error("Duplicate configuration keyword"));
            }
            *entry = Some(name(&mut parser)?);
            continue;
        }
        let list = match keyword.text.as_str() {
            "INVARIANT" | "INVARIANTS" => Some(&mut config.invariants),
            "PROPERTY" | "PROPERTIES" => Some(&mut config.properties),
            "CONSTRAINT" | "CONSTRAINTS" => Some(&mut config.constraints),
            "ACTION_CONSTRAINT" | "ACTION_CONSTRAINTS" => Some(&mut config.action_constraints),
            "POSTCONDITION" | "POSTCONDITIONS" => Some(&mut config.postconditions),
            "_POSSIBLE" => Some(&mut config.possible),
            _ => None,
        };
        if let Some(entries) = list {
            while parser.raw().kind != Kind::End && !KEYWORDS.contains(&parser.raw().text.as_str())
            {
                entries.push(name(&mut parser)?);
            }
            continue;
        }
        match keyword.text.as_str() {
            "CHECK_DEADLOCK" => {
                if deadlock_set || (!parser.at("TRUE") && !parser.at("FALSE")) {
                    return Err(parser.error("Expected one CHECK_DEADLOCK TRUE or FALSE"));
                }
                config.check_deadlock = parser.advance()?.text == "TRUE";
                deadlock_set = true;
            }
            "CONSTANT" | "CONSTANTS" => {
                while parser.raw().kind != Kind::End
                    && !KEYWORDS.contains(&parser.raw().text.as_str())
                {
                    let start = parser.cursor;
                    let name = name(&mut parser)?;
                    if parser.accept("<-") {
                        let module = scope(&mut parser)?;
                        let target = self::name(&mut parser)?;
                        config.substitutions.push(Substitution {
                            name,
                            target,
                            module,
                        });
                    } else {
                        let args = if parser.accept("(") {
                            let mut args = vec![value(&mut parser, 0)?];
                            while parser.accept(",") {
                                args.push(value(&mut parser, 0)?);
                            }
                            parser.expect(")")?;
                            Some(args)
                        } else {
                            None
                        };
                        parser.expect("=")?;
                        let module = scope(&mut parser)?;
                        config.constants.push(ConstantAssignment {
                            name,
                            args,
                            module,
                            value: value(&mut parser, 0)?,
                        });
                    }
                    config.raw_constants.push(
                        parser.tokens[start..parser.cursor]
                            .iter()
                            .map(|token| token.raw.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                }
            }
            _ => return Err(parser.error("Unknown configuration keyword")),
        }
    }
    Ok(config)
}
