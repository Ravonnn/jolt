use std::collections::HashMap;

use jolt_source::Span;

/// Stable symbol id for a binding site (for later passes).
pub type SymbolId = u32;

/// Where a name was introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingOrigin {
    Param,
    ImmutableBinding,
    MutableBinding,
    ForLoop,
}

/// A resolved binding in scope.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BindingInfo {
    pub id: SymbolId,
    pub name: String,
    pub mutable: bool,
    pub span: Span,
    pub origin: BindingOrigin,
}

struct Scope {
    bindings: HashMap<String, BindingInfo>,
}

/// Lexical scope stack for Tiny name resolution.
pub struct ScopeStack {
    scopes: Vec<Scope>,
    next_id: SymbolId,
    pub symbols: Vec<BindingInfo>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                bindings: HashMap::new(),
            }],
            next_id: 0,
            symbols: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            bindings: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    pub fn contains_in_current(&self, name: &str) -> bool {
        self.scopes
            .last()
            .is_some_and(|s| s.bindings.contains_key(name))
    }

    pub fn lookup(&self, name: &str) -> Option<&BindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.bindings.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn declare(
        &mut self,
        name: impl Into<String>,
        mutable: bool,
        span: Span,
        origin: BindingOrigin,
    ) -> BindingInfo {
        let name = name.into();
        let id = self.next_id;
        self.next_id += 1;
        let info = BindingInfo {
            id,
            name: name.clone(),
            mutable,
            span,
            origin,
        };
        self.scopes
            .last_mut()
            .expect("scope stack")
            .bindings
            .insert(name, info.clone());
        self.symbols.push(info.clone());
        info
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}
