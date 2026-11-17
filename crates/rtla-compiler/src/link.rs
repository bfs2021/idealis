use crate::rewrite::{Names, Rewriter, identifier};
use crate::{CompileError, Result, unsupported};
use rtla::ast::{
    Assumption, CaseArm, Config, ConfigValue, ConstantAssignment, Declaration, Definition,
    DefinitionKind, Expression, ExpressionKind as E, Location, Module,
};
use std::collections::{BTreeMap, BTreeSet};

const STANDARD: &[&str] = &[
    "Naturals",
    "Integers",
    "Sequences",
    "FiniteSets",
    "TLC",
    "Apalache",
    "Variants",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Constant,
    Variable,
    Definition,
}

#[derive(Clone)]
struct Symbol {
    id: String,
    kind: Kind,
    arity: usize,
    local: bool,
}

#[derive(Clone, Default)]
struct Linked {
    symbols: BTreeMap<String, Symbol>,
    definitions: BTreeMap<String, Definition>,
    origins: BTreeMap<String, (String, String)>,
    constants: BTreeMap<String, Declaration>,
    variables: BTreeMap<String, Declaration>,
    assumptions: BTreeMap<String, Assumption>,
    extends: BTreeSet<String>,
}

impl Linked {
    fn insert(&mut self, name: &str, mut symbol: Symbol) -> Result<()> {
        if self
            .symbols
            .get(name)
            .is_some_and(|previous| previous.id != symbol.id)
        {
            return unsupported(format!("conflicting module symbol {name}"));
        }
        if let Some(previous) = self.symbols.get(name) {
            symbol.local &= previous.local;
        }
        self.symbols.insert(name.into(), symbol);
        Ok(())
    }

    fn merge(&mut self, dependency: &Self) -> Result<()> {
        for (name, symbol) in &dependency.symbols {
            if !symbol.local {
                self.insert(name, symbol.clone())?;
            }
        }
        self.definitions.extend(dependency.definitions.clone());
        self.origins.extend(dependency.origins.clone());
        self.constants.extend(dependency.constants.clone());
        self.variables.extend(dependency.variables.clone());
        self.assumptions.extend(dependency.assumptions.clone());
        self.extends.extend(dependency.extends.clone());
        self.limit()
    }

    fn limit(&self) -> Result<()> {
        if self.definitions.len() + self.constants.len() + self.variables.len() > 4096 {
            return unsupported("module expansion exceeds 4096 symbols");
        }
        Ok(())
    }

    fn names(&self, location: Location) -> Names {
        self.symbols
            .iter()
            .map(|(name, symbol)| (name.clone(), identifier(&symbol.id, location)))
            .collect()
    }
}

struct Linker<'a> {
    sources: BTreeMap<String, &'a Module>,
    cache: BTreeMap<String, Linked>,
    visiting: Vec<String>,
    rewrite: Rewriter,
}

impl Linker<'_> {
    fn module(&mut self, name: &str) -> Result<Linked> {
        if self.visiting.iter().any(|entry| entry == name) {
            return unsupported(format!("cyclic module import {name}"));
        }
        if let Some(module) = self.cache.get(name) {
            return Ok(module.clone());
        }
        if self.visiting.len() >= 40 {
            return unsupported("module import depth exceeds 40");
        }
        let module = *self.sources.get(name).ok_or_else(|| CompileError {
            message: format!("missing module {name}"),
        })?;
        self.visiting.push(name.into());
        let mut result = Linked::default();
        for dependency in &module.extends {
            if STANDARD.contains(&dependency.as_str()) {
                result.extends.insert(dependency.clone());
            } else {
                result.merge(&self.module(dependency)?)?;
            }
        }
        let mut imports = result.extends.clone();
        for (kind, declarations) in [
            (Kind::Constant, &module.constants),
            (Kind::Variable, &module.variables),
        ] {
            for declaration in declarations {
                let id = self.rewrite.fresh();
                result.insert(
                    &declaration.name,
                    Symbol {
                        id: id.clone(),
                        kind,
                        arity: declaration.arity,
                        local: false,
                    },
                )?;
                let mut declaration = declaration.clone();
                declaration.name = id.clone();
                if kind == Kind::Constant {
                    result.constants.insert(id, declaration);
                } else {
                    result.variables.insert(id, declaration);
                }
            }
        }
        for definition in &module.definitions {
            let arity = match &definition.head {
                DefinitionKind::Operator { params } => params.len(),
                DefinitionKind::Function { .. } => 0,
            };
            result.insert(
                &definition.name,
                Symbol {
                    id: self.rewrite.fresh(),
                    kind: Kind::Definition,
                    arity,
                    local: definition.is_local,
                },
            )?;
        }
        for instance in &module.instances {
            let template = self.module(&instance.module)?;
            if instance.name.is_none() {
                imports.extend(template.extends.clone());
                if template.constants.is_empty()
                    && template.variables.is_empty()
                    && instance.substitutions.is_empty()
                {
                    let mut shared = template.clone();
                    shared.assumptions.clear();
                    for symbol in shared.symbols.values_mut() {
                        symbol.local |= instance.is_local;
                    }
                    result.merge(&shared)?;
                    for (name, symbol) in &template.symbols {
                        if !symbol.local {
                            result.insert(
                                name,
                                Symbol {
                                    local: instance.is_local,
                                    ..symbol.clone()
                                },
                            )?;
                        }
                    }
                    continue;
                }
            }
            let mut replacements = BTreeMap::new();
            for substitution in &instance.substitutions {
                if !template
                    .symbols
                    .get(&substitution.name)
                    .is_some_and(|symbol| symbol.kind != Kind::Definition)
                {
                    return unsupported(format!(
                        "unknown INSTANCE parameter {}",
                        substitution.name
                    ));
                }
                if replacements
                    .insert(substitution.name.clone(), substitution.expression.clone())
                    .is_some()
                {
                    return unsupported("duplicate INSTANCE substitution");
                }
            }
            let names = result.names(instance.location);
            let mut mapping = Names::new();
            for (name, symbol) in &template.symbols {
                if symbol.kind == Kind::Definition {
                    continue;
                }
                if symbol.arity != 0 {
                    return unsupported("higher order INSTANCE parameter");
                }
                let mut expression = replacements
                    .remove(name)
                    .unwrap_or_else(|| identifier(name, instance.location));
                if symbol.kind == Kind::Variable
                    && !matches!(&expression.node, E::Identifier { name } if result.symbols.get(name).is_some_and(|symbol| symbol.kind == Kind::Variable))
                {
                    return unsupported("INSTANCE variable substitution requires a state variable");
                }
                self.rewrite.strict = true;
                self.rewrite.imports = imports.clone();
                self.rewrite.expression(&mut expression, &names)?;
                self.rewrite.strict = false;
                if matches!(&expression.node, E::Identifier { name } if !result.symbols.values().any(|symbol| symbol.id == *name) && name != "TRUE" && name != "FALSE" && name != "BOOLEAN")
                {
                    return unsupported(format!("missing INSTANCE substitution {name}"));
                }
                mapping.insert(symbol.id.clone(), expression);
            }
            for id in template.definitions.keys() {
                mapping.insert(
                    id.clone(),
                    identifier(&self.rewrite.fresh(), instance.location),
                );
            }
            for (id, definition) in &template.definitions {
                let E::Identifier { name } = &mapping[id].node else {
                    unreachable!()
                };
                let mut definition = definition.clone();
                definition.name = name.clone();
                self.rewrite.definition(&mut definition, &mapping)?;
                result.definitions.insert(name.clone(), definition);
                result
                    .origins
                    .insert(name.clone(), template.origins[id].clone());
            }
            for (name, symbol) in &template.symbols {
                if symbol.local || symbol.kind != Kind::Definition {
                    continue;
                }
                let E::Identifier { name: id } = &mapping[&symbol.id].node else {
                    unreachable!()
                };
                let public = instance
                    .name
                    .as_ref()
                    .map_or_else(|| name.clone(), |prefix| format!("{prefix}!{name}"));
                result.insert(
                    &public,
                    Symbol {
                        id: id.clone(),
                        local: instance.is_local,
                        ..symbol.clone()
                    },
                )?;
            }
            result.extends.extend(template.extends);
            result.limit()?;
        }
        let names = result.names(module.location);
        self.rewrite.strict = true;
        self.rewrite.imports = imports;
        for definition in &module.definitions {
            let id = result.symbols[&definition.name].id.clone();
            let mut rewritten = definition.clone();
            rewritten.name = id.clone();
            self.rewrite.definition(&mut rewritten, &names)?;
            result.definitions.insert(id.clone(), rewritten);
            result
                .origins
                .insert(id, (module.name.clone(), definition.name.clone()));
        }
        for assumption in &module.assumptions {
            let mut assumption = assumption.clone();
            self.rewrite
                .expression(&mut assumption.expression, &names)?;
            result.assumptions.insert(self.rewrite.fresh(), assumption);
        }
        self.rewrite.strict = false;
        result.limit()?;
        self.visiting.pop();
        self.cache.insert(name.into(), result.clone());
        Ok(result)
    }

    fn literal(
        &mut self,
        value: &ConfigValue,
        location: Location,
        linked: &mut Linked,
        config: &mut Config,
    ) -> Expression {
        let name = self.rewrite.fresh();
        linked.constants.insert(
            name.clone(),
            Declaration {
                name: name.clone(),
                arity: 0,
                location,
            },
        );
        config.constants.push(ConstantAssignment {
            name: name.clone(),
            value: value.clone(),
            args: None,
            module: None,
        });
        identifier(&name, location)
    }

    fn targets(&self, linked: &Linked, name: &str, module: Option<&str>) -> Result<Vec<Symbol>> {
        if let Some(module) = module {
            let ids = linked
                .origins
                .iter()
                .filter(|(_, origin)| origin.0 == module && origin.1 == name)
                .map(|(id, _)| id);
            let mut symbols = vec![];
            for id in ids {
                let definition = &linked.definitions[id];
                let arity = match &definition.head {
                    DefinitionKind::Operator { params } => params.len(),
                    DefinitionKind::Function { .. } => 0,
                };
                symbols.push(Symbol {
                    id: id.clone(),
                    arity,
                    kind: Kind::Definition,
                    local: false,
                });
            }
            if symbols.is_empty() {
                return unsupported(format!("unknown module override {module}!{name}"));
            }
            Ok(symbols)
        } else {
            Ok(vec![
                linked
                    .symbols
                    .get(name)
                    .cloned()
                    .filter(|symbol| symbol.kind != Kind::Variable)
                    .ok_or_else(|| CompileError {
                        message: format!("unknown configuration symbol {name}"),
                    })?,
            ])
        }
    }

    fn configure(
        &mut self,
        linked: &mut Linked,
        input: &Config,
        location: Location,
    ) -> Result<Config> {
        let mut config = input.clone();
        config.constants.clear();
        config.substitutions.clear();
        let mut assigned = BTreeMap::<String, Vec<&ConstantAssignment>>::new();
        for assignment in &input.constants {
            for symbol in self.targets(linked, &assignment.name, assignment.module.as_deref())? {
                assigned.entry(symbol.id).or_default().push(assignment);
            }
        }
        for (id, assignments) in &assigned {
            if linked
                .constants
                .get(id)
                .is_some_and(|declaration| declaration.arity != 0)
            {
                return unsupported(
                    "constant operator tables require a defined operator; use <- for operator declarations",
                );
            }
            let arity = linked
                .constants
                .get(id)
                .map(|declaration| declaration.arity)
                .unwrap_or_else(|| match &linked.definitions[id].head {
                    DefinitionKind::Operator { params } => params.len(),
                    DefinitionKind::Function { .. } => 0,
                });
            if arity == 0
                && assignments.len() == 1
                && assignments[0].args.is_none()
                && linked.constants.contains_key(id)
            {
                let mut assignment = assignments[0].clone();
                assignment.name = id.clone();
                assignment.module = None;
                config.constants.push(assignment);
                continue;
            }
            let params: Vec<_> = (0..arity)
                .map(|_| Declaration {
                    name: self.rewrite.fresh(),
                    arity: 0,
                    location,
                })
                .collect();
            let mut arms = vec![];
            let mut keys = BTreeSet::new();
            for assignment in assignments {
                let args = assignment.args.clone().unwrap_or_default();
                if args.len() != arity || !keys.insert(serde_json::to_string(&args).unwrap()) {
                    return unsupported("duplicate or invalid constant table arguments");
                }
                let mut operands = vec![];
                for (param, value) in params.iter().zip(&args) {
                    operands.push(Expression {
                        node: E::Apply {
                            operator: "=".into(),
                            args: vec![
                                identifier(&param.name, location),
                                self.literal(value, location, linked, &mut config),
                            ],
                        },
                        location,
                    });
                }
                arms.push(CaseArm {
                    condition: Expression {
                        node: E::And {
                            operands,
                            is_bulleted: false,
                        },
                        location,
                    },
                    body: self.literal(&assignment.value, location, linked, &mut config),
                });
            }
            linked.constants.remove(id);
            linked.definitions.insert(
                id.clone(),
                Definition {
                    name: id.clone(),
                    head: DefinitionKind::Operator { params },
                    body: Expression {
                        node: E::Case { arms, other: None },
                        location,
                    },
                    is_local: false,
                    is_recursive: false,
                    location,
                },
            );
        }
        let mut overrides = vec![];
        let mut replaced = BTreeSet::new();
        for substitution in &input.substitutions {
            let target = linked
                .symbols
                .get(&substitution.target)
                .filter(|symbol| symbol.kind == Kind::Definition)
                .cloned()
                .ok_or_else(|| CompileError {
                    message: format!("unknown override target {}", substitution.target),
                })?;
            for symbol in
                self.targets(linked, &substitution.name, substitution.module.as_deref())?
            {
                if symbol.arity != target.arity
                    || assigned.contains_key(&symbol.id)
                    || !replaced.insert(symbol.id.clone())
                {
                    return unsupported("duplicate override or wrong operator arity");
                }
                overrides.push((symbol, target.clone()));
            }
        }
        for (symbol, target) in overrides {
            if replaced.contains(&target.id) {
                return unsupported("override target is itself overridden");
            }
            let params: Vec<_> = (0..symbol.arity)
                .map(|_| Declaration {
                    name: self.rewrite.fresh(),
                    arity: 0,
                    location,
                })
                .collect();
            let body = if params.is_empty() {
                identifier(&target.id, location)
            } else {
                Expression {
                    node: E::Apply {
                        operator: target.id,
                        args: params
                            .iter()
                            .map(|param| identifier(&param.name, location))
                            .collect(),
                    },
                    location,
                }
            };
            linked.constants.remove(&symbol.id);
            linked.definitions.insert(
                symbol.id.clone(),
                Definition {
                    name: symbol.id,
                    head: DefinitionKind::Operator { params },
                    body,
                    is_local: false,
                    is_recursive: false,
                    location,
                },
            );
        }
        linked.limit()?;
        Ok(config)
    }
}

pub(crate) fn resolve(
    root: &Module,
    config: &Config,
    modules: &[Module],
) -> Result<(Module, Config, BTreeMap<String, String>)> {
    if modules.len() > 256 {
        return unsupported("source bundle exceeds 256 modules");
    }
    let mut linker = Linker {
        sources: BTreeMap::new(),
        cache: BTreeMap::new(),
        visiting: vec![],
        rewrite: Rewriter {
            next: 0,
            strict: false,
            imports: BTreeSet::new(),
            labels: BTreeMap::new(),
            depth: 0,
        },
    };
    for module in std::iter::once(root).chain(modules) {
        if STANDARD.contains(&module.name.as_str())
            || linker.sources.insert(module.name.clone(), module).is_some()
        {
            return unsupported(format!("duplicate or reserved module {}", module.name));
        }
    }
    let mut linked = linker.module(&root.name)?;
    let config = linker.configure(&mut linked, config, root.location)?;
    let mut names = Names::new();
    let mut variables = vec![];
    for (name, symbol) in &linked.symbols {
        if symbol.kind == Kind::Variable {
            names.insert(symbol.id.clone(), identifier(name, root.location));
            let mut declaration = linked.variables[&symbol.id].clone();
            declaration.name = name.clone();
            variables.push(declaration);
        }
    }
    for definition in linked.definitions.values_mut() {
        linker.rewrite.definition(definition, &names)?;
    }
    for assumption in linked.assumptions.values_mut() {
        linker
            .rewrite
            .expression(&mut assumption.expression, &names)?;
    }
    for (name, symbol) in &linked.symbols {
        if let Some(definition) = linked.definitions.get(&symbol.id) {
            let mut alias = definition.clone();
            alias.name = name.clone();
            alias.body = match &alias.head {
                DefinitionKind::Operator { params } if !params.is_empty() => Expression {
                    node: E::Apply {
                        operator: symbol.id.clone(),
                        args: params
                            .iter()
                            .map(|param| identifier(&param.name, param.location))
                            .collect(),
                    },
                    location: alias.location,
                },
                _ => identifier(&symbol.id, alias.location),
            };
            if matches!(alias.head, DefinitionKind::Function { .. }) {
                alias.head = DefinitionKind::Operator { params: vec![] };
            }
            linked.definitions.insert(name.clone(), alias);
        }
    }
    linked.limit()?;
    let mut action_names = linker.rewrite.labels;
    action_names.extend(
        linked
            .origins
            .iter()
            .map(|(id, (module, name))| (id.clone(), format!("{module}!{name}"))),
    );
    for (name, symbol) in &linked.symbols {
        if linked.definitions.contains_key(&symbol.id) {
            action_names.insert(symbol.id.clone(), name.clone());
        }
    }
    Ok((
        Module {
            name: root.name.clone(),
            extends: linked.extends.into_iter().collect(),
            constants: linked.constants.into_values().collect(),
            variables,
            definitions: linked.definitions.into_values().collect(),
            instances: vec![],
            assumptions: linked.assumptions.into_values().collect(),
            theorems: vec![],
            location: root.location,
        },
        config,
        action_names,
    ))
}
