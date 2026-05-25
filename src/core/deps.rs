use std::collections::HashMap;

use super::models::{DepKind, DepNode, Package};

/// Build a (lazily-truncated) dependency tree rooted at `pkg`. `index` maps
/// names to known package metadata (repo + AUR + installed). Anything not in
/// the index is rendered as a leaf with `installed=false`.
pub fn build_tree(pkg: &Package, index: &HashMap<String, Package>, max_depth: usize) -> DepNode {
    let mut node = DepNode {
        name: pkg.name.clone(),
        kind: DepKind::Required,
        installed: pkg.installed,
        children: Vec::new(),
    };
    expand(&mut node, pkg, index, max_depth, &mut Vec::new());
    node
}

fn expand(
    parent: &mut DepNode,
    pkg: &Package,
    index: &HashMap<String, Package>,
    depth_left: usize,
    visited: &mut Vec<String>,
) {
    if depth_left == 0 {
        return;
    }
    visited.push(pkg.name.clone());

    for (deps, kind) in [
        (&pkg.depends, DepKind::Required),
        (&pkg.make_depends, DepKind::Make),
        (&pkg.opt_depends, DepKind::Optional),
    ] {
        for raw in deps {
            let name = strip_constraint(raw);
            if name.is_empty() || visited.iter().any(|v| v == &name) {
                continue;
            }
            let (installed, child_pkg) = match index.get(&name) {
                Some(p) => (p.installed, Some(p)),
                None => (false, None),
            };
            let mut child = DepNode {
                name,
                kind,
                installed,
                children: Vec::new(),
            };
            if let Some(p) = child_pkg {
                expand(&mut child, p, index, depth_left - 1, visited);
            }
            parent.children.push(child);
        }
    }
    visited.pop();
}

/// "foo>=1.0: optional message" → "foo".
fn strip_constraint(raw: &str) -> String {
    let mut end = raw.len();
    for (i, c) in raw.char_indices() {
        if matches!(c, '<' | '>' | '=' | ':' | ' ') {
            end = i;
            break;
        }
    }
    raw[..end].to_string()
}

/// Walk all installed packages and collect names that list `target` in their
/// `depends`. Cheap N×M scan — fine for typical Arch installs (~1500 pkgs).
pub fn reverse_deps(target: &str, installed: &[Package]) -> Vec<String> {
    installed
        .iter()
        .filter(|p| {
            p.depends
                .iter()
                .any(|d| strip_constraint(d) == target)
        })
        .map(|p| p.name.clone())
        .collect()
}
