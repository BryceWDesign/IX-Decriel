//! Abstract syntax tree model for Decriel.
//!
//! The AST is the parsed structural form of Decriel source. Wave 1 uses this
//! model for syntax recognition only; later waves will attach capability
//! semantics, effect tracking, policy validation, runtime enforcement, evidence,
//! secret tracking, supply-chain security, cryptographic APIs, privacy profiles,
//! proof obligations, and backend lowering.

use decriel_diagnostics::SourceSpan;

/// Source-backed identifier used by Decriel declarations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Identifier {
    name: String,
    span: SourceSpan,
}

impl Identifier {
    /// Creates an identifier.
    #[must_use]
    pub fn new(name: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span for the identifier.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

/// Parsed Decriel module.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DecrielModule {
    name: Identifier,
    declarations: Vec<Declaration>,
    span: SourceSpan,
}

impl DecrielModule {
    /// Creates a parsed Decriel module.
    #[must_use]
    pub fn new(name: Identifier, declarations: Vec<Declaration>, span: SourceSpan) -> Self {
        Self {
            name,
            declarations,
            span,
        }
    }

    /// Returns the module name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns parsed declarations inside the module body.
    #[must_use]
    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    /// Returns the module source span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns true when this module contains no declarations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Returns the number of parsed declarations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.declarations.len()
    }
}

/// Top-level declaration inside a Decriel module.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Declaration {
    /// Capability declaration shell.
    Capability {
        /// Capability action.
        action: CapabilityAction,
        /// Capability target.
        target: Identifier,
        /// Declaration source span.
        span: SourceSpan,
    },
    /// Effect declaration shell.
    Effect {
        /// Effect action.
        action: EffectAction,
        /// Effect target.
        target: Identifier,
        /// Declaration source span.
        span: SourceSpan,
    },
    /// Policy declaration shell.
    Policy {
        /// Policy action.
        action: PolicyAction,
        /// Policy target.
        target: Identifier,
        /// Declaration source span.
        span: SourceSpan,
    },
    /// Function declaration shell.
    Function {
        /// Function name.
        name: Identifier,
        /// Declaration source span.
        span: SourceSpan,
    },
}

impl Declaration {
    /// Returns the source span for this declaration.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::Capability { span, .. }
            | Self::Effect { span, .. }
            | Self::Policy { span, .. }
            | Self::Function { span, .. } => *span,
        }
    }

    /// Returns a stable declaration kind name for diagnostics and AST output.
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Capability { .. } => "capability",
            Self::Effect { .. } => "effect",
            Self::Policy { .. } => "policy",
            Self::Function { .. } => "function",
        }
    }
}

/// Capability declaration action.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CapabilityAction {
    /// Read authority.
    Read,
    /// Write authority.
    Write,
    /// Network authority.
    Network,
    /// Execute authority.
    Execute,
    /// Secret authority.
    Secret,
}

impl CapabilityAction {
    /// Returns the canonical source spelling for this capability action.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Network => "network",
            Self::Execute => "execute",
            Self::Secret => "secret",
        }
    }
}

/// Effect declaration action.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EffectAction {
    /// File read effect.
    Read,
    /// File write effect.
    Write,
    /// Network effect.
    Network,
    /// Trace output effect.
    Trace,
    /// Evidence output effect.
    Evidence,
}

impl EffectAction {
    /// Returns the canonical source spelling for this effect action.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Network => "network",
            Self::Trace => "trace",
            Self::Evidence => "evidence",
        }
    }
}

/// Policy declaration action.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PolicyAction {
    /// Allow a declared operation.
    Allow,
    /// Deny a declared operation.
    Deny,
    /// Require a declared condition.
    Requires,
    /// Ensure a declared guarantee.
    Ensures,
}

impl PolicyAction {
    /// Returns the canonical source spelling for this policy action.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Requires => "requires",
            Self::Ensures => "ensures",
        }
    }
}

#[cfg(test)]
mod tests {
    use decriel_diagnostics::{SourceLocation, SourceSpan};

    use super::{
        CapabilityAction, Declaration, DecrielModule, EffectAction, Identifier, PolicyAction,
    };

    fn span() -> SourceSpan {
        SourceSpan::new(SourceLocation::new(1, 1), SourceLocation::new(1, 8))
    }

    #[test]
    fn identifier_preserves_name_and_span() {
        let identifier = Identifier::new("secure_service", span());

        assert_eq!(identifier.name(), "secure_service");
        assert_eq!(identifier.span(), span());
    }

    #[test]
    fn module_reports_declarations() {
        let name = Identifier::new("main", span());
        let declaration = Declaration::Function {
            name: Identifier::new("entry", span()),
            span: span(),
        };
        let module = DecrielModule::new(name, vec![declaration], span());

        assert_eq!(module.name().name(), "main");
        assert_eq!(module.len(), 1);
        assert!(!module.is_empty());
        assert_eq!(module.declarations()[0].kind_name(), "function");
    }

    #[test]
    fn declaration_kind_names_are_stable() {
        let target = Identifier::new("network", span());

        let capability = Declaration::Capability {
            action: CapabilityAction::Network,
            target: target.clone(),
            span: span(),
        };
        let effect = Declaration::Effect {
            action: EffectAction::Trace,
            target: target.clone(),
            span: span(),
        };
        let policy = Declaration::Policy {
            action: PolicyAction::Deny,
            target,
            span: span(),
        };

        assert_eq!(capability.kind_name(), "capability");
        assert_eq!(effect.kind_name(), "effect");
        assert_eq!(policy.kind_name(), "policy");
    }

    #[test]
    fn action_source_spelling_is_stable() {
        assert_eq!(CapabilityAction::Read.as_str(), "read");
        assert_eq!(CapabilityAction::Network.as_str(), "network");
        assert_eq!(EffectAction::Evidence.as_str(), "evidence");
        assert_eq!(PolicyAction::Requires.as_str(), "requires");
    }
}
