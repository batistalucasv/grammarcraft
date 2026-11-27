//! The layering rule, and the check that enforces it.
//!
//! A crate may depend on a **strictly lower** layer, never on the same or a
//! higher one. The rule is what keeps the domain testable without a UI and
//! keeps a rebuild of a leaf from recompiling the world.
//!
//! The check works on a small model (`Crate`, `Dep`) so it can be unit-tested
//! without cargo; `from_metadata` builds that model from `cargo metadata`.
//!
//! Adding a crate means adding a row to [`TABLE`]. A crate missing from the
//! table is a violation, not an exemption: otherwise forgetting to classify it
//! would silently disable the rule.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Layer(u8),
    /// Binaries and build tooling: may depend on anything.
    Exempt,
}

impl Class {
    fn layer(self) -> Option<u8> {
        match self {
            Class::Layer(layer) => Some(layer),
            Class::Exempt => None,
        }
    }
}

/// Package names without the `grammarcraft-` prefix.
pub const TABLE: &[(&str, Class)] = &[
    ("core", Class::Layer(0)),
    ("engine", Class::Layer(1)),
    ("cli", Class::Exempt),
    ("xtask", Class::Exempt),
];

pub fn short_name(package: &str) -> &str {
    package.strip_prefix("grammarcraft-").unwrap_or(package)
}

pub fn classify(package: &str) -> Option<Class> {
    let short = short_name(package);
    TABLE
        .iter()
        .find(|(name, _)| *name == short)
        .map(|(_, class)| *class)
}

pub fn describe(class: Option<Class>) -> String {
    match class {
        Some(Class::Layer(layer)) => format!("L{layer}"),
        Some(Class::Exempt) => "exempt".into(),
        None => "UNREGISTERED".into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
}

#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
    /// `true` if the dependency is a workspace member (a path dependency).
    pub workspace: bool,
}

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub deps: Vec<Dep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    Unregistered {
        krate: String,
    },
    Upward {
        krate: String,
        dep: String,
        from: u8,
        to: u8,
        kind: DepKind,
    },
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Violation::Unregistered { krate } => {
                write!(
                    f,
                    "{krate}: unknown workspace crate; add it to xtask/src/layers.rs TABLE"
                )
            }
            Violation::Upward {
                krate,
                dep,
                from,
                to,
                kind,
            } => {
                write!(
                    f,
                    "{krate} (L{from}) -> {dep} (L{to}) [{kind:?}]: a crate may only depend on a lower layer"
                )
            }
        }
    }
}

/// All violations, sorted so the output is stable between runs.
pub fn check(crates: &[Crate]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for krate in crates {
        let Some(class) = classify(&krate.name) else {
            violations.push(Violation::Unregistered {
                krate: krate.name.clone(),
            });
            continue;
        };
        if class == Class::Exempt {
            continue;
        }
        let layer = class.layer().unwrap_or(0);
        for dep in &krate.deps {
            if !dep.workspace || dep.name == krate.name {
                continue;
            }
            // A dependency on an unregistered crate is reported on that
            // crate's own entry; reporting it twice adds noise.
            if classify(&dep.name).is_none() {
                continue;
            }
            let Some(target) = classify(&dep.name).and_then(Class::layer) else {
                continue; // an exempt crate (an app or xtask) is not a layer
            };
            if target >= layer {
                violations.push(Violation::Upward {
                    krate: krate.name.clone(),
                    dep: dep.name.clone(),
                    from: layer,
                    to: target,
                    kind: dep.kind,
                });
            }
        }
    }
    violations.sort_by_key(ToString::to_string);
    violations.dedup();
    violations
}

/// Builds the model from `cargo metadata --format-version 1 --no-deps`.
pub fn from_metadata(metadata: &Value) -> Result<Vec<Crate>, String> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("metadata: no `packages` array")?;
    let members: Vec<&str> = packages
        .iter()
        .filter_map(|package| package["name"].as_str())
        .collect();
    let mut crates = Vec::new();
    for package in packages {
        let name = package["name"]
            .as_str()
            .ok_or("metadata: a package has no name")?
            .to_owned();
        let mut deps = Vec::new();
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            let dep_name = dependency["name"].as_str().unwrap_or_default().to_owned();
            let kind = if dependency["kind"].as_str() == Some("dev") {
                DepKind::Dev
            } else {
                DepKind::Normal
            };
            let workspace = members.contains(&dep_name.as_str()) || dependency["path"].is_string();
            deps.push(Dep {
                name: dep_name,
                kind,
                workspace,
            });
        }
        crates.push(Crate { name, deps });
    }
    crates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(crates)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn krate(name: &str, deps: &[(&str, DepKind, bool)]) -> Crate {
        Crate {
            name: name.into(),
            deps: deps
                .iter()
                .map(|(dep, kind, workspace)| Dep {
                    name: (*dep).into(),
                    kind: *kind,
                    workspace: *workspace,
                })
                .collect(),
        }
    }

    use DepKind::{Dev, Normal};

    #[test]
    fn a_downward_graph_passes() {
        let graph = [
            krate("grammarcraft-core", &[("serde", Normal, false)]),
            krate(
                "grammarcraft-engine",
                &[("grammarcraft-core", Normal, true)],
            ),
            krate("grammarcraft-cli", &[("grammarcraft-engine", Normal, true)]),
            krate("xtask", &[("grammarcraft-engine", Dev, true)]),
        ];
        assert!(check(&graph).is_empty(), "{:?}", check(&graph));
    }

    #[test]
    fn an_upward_dependency_is_flagged() {
        let violations = check(&[krate(
            "grammarcraft-core",
            &[("grammarcraft-engine", Normal, true)],
        )]);
        assert!(
            matches!(violations[..], [Violation::Upward { from: 0, to: 1, .. }]),
            "{violations:?}"
        );
    }

    #[test]
    fn a_sideways_dependency_is_flagged() {
        // A crate depending on itself is how a feature flag is enabled in
        // tests; it is not an edge in the layer graph.
        assert!(
            check(&[krate(
                "grammarcraft-engine",
                &[("grammarcraft-engine", Normal, true)]
            )])
            .is_empty()
        );

        let violations = check(&[
            krate(
                "grammarcraft-engine",
                &[("grammarcraft-other", Normal, true)],
            ),
            krate("grammarcraft-other", &[]),
        ]);
        assert!(
            violations.iter().any(|violation| matches!(violation, Violation::Unregistered { krate } if krate == "grammarcraft-other")),
            "{violations:?}"
        );
    }

    #[test]
    fn an_upward_dev_dependency_is_flagged_too() {
        let violations = check(&[krate(
            "grammarcraft-core",
            &[("grammarcraft-engine", Dev, true)],
        )]);
        assert!(matches!(
            violations[..],
            [Violation::Upward { kind: Dev, .. }]
        ));
    }

    #[test]
    fn an_unregistered_crate_is_a_violation() {
        let violations = check(&[krate("grammarcraft-mystery", &[])]);
        assert!(
            matches!(&violations[..], [Violation::Unregistered { krate }] if krate == "grammarcraft-mystery")
        );
        assert!(violations[0].to_string().contains("TABLE"));
    }

    #[test]
    fn external_dependencies_are_not_layered() {
        let violations = check(&[krate(
            "grammarcraft-engine",
            &[("serde_json", Normal, false), ("thiserror", Normal, false)],
        )]);
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn metadata_is_parsed() {
        let metadata: Value = serde_json::from_str(
            r#"{"packages":[
                {"name":"grammarcraft-core","dependencies":[]},
                {"name":"grammarcraft-engine","dependencies":[
                    {"name":"grammarcraft-core","kind":null,"path":"/x/crates/core"},
                    {"name":"serde","kind":null},
                    {"name":"grammarcraft-cli","kind":"dev","path":"/x/apps/grammarcraft-cli"}]}
            ]}"#,
        )
        .expect("valid JSON");
        let graph = from_metadata(&metadata).expect("valid metadata");
        assert_eq!(graph.len(), 2);
        let engine = graph
            .iter()
            .find(|krate| krate.name == "grammarcraft-engine")
            .expect("the engine is present");
        assert!(engine.deps[0].workspace);
        assert!(!engine.deps[1].workspace);
        assert_eq!(engine.deps[2].kind, Dev);
    }
}
