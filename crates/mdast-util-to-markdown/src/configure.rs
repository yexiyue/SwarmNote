use std::collections::HashMap;

use crate::state::State;
use crate::types::{HandlerFn, JoinFn, Options, UnsafePattern};

/// Input extension for configuring the serializer.
///
/// This is the Rust equivalent of the JS `Options` type when used as an extension.
/// Fields are all optional - only set fields will be merged.
pub struct InputExtension {
    /// Handler overrides.
    pub handlers: Option<HashMap<String, HandlerFn>>,
    /// Additional unsafe patterns.
    pub unsafe_patterns: Option<Vec<UnsafePattern>>,
    /// Additional join functions.
    pub join: Option<Vec<JoinFn>>,
    /// Sub-extensions to merge first.
    pub extensions: Option<Vec<InputExtension>>,
    /// Options overrides.
    pub options: Option<OptionsOverride>,
}

/// Optional overrides for serialization options.
///
/// Each field is `Option<T>` so only explicitly set values are merged.
#[derive(Default)]
pub struct OptionsOverride {
    pub bullet: Option<char>,
    pub bullet_other: Option<char>,
    pub bullet_ordered: Option<char>,
    pub close_atx: Option<bool>,
    pub emphasis: Option<char>,
    pub fence: Option<char>,
    pub fences: Option<bool>,
    pub increment_list_marker: Option<bool>,
    pub list_item_indent: Option<String>,
    pub quote: Option<char>,
    pub resource_link: Option<bool>,
    pub rule: Option<char>,
    pub rule_repetition: Option<usize>,
    pub rule_spaces: Option<bool>,
    pub setext: Option<bool>,
    pub strong: Option<char>,
    pub tight_definitions: Option<bool>,
}

/// Merge an extension into the state.
///
/// Port of JS `configure()` from `lib/configure.js`.
///
/// Merging semantics:
/// - `extensions`: recursively merged first
/// - `handlers`: last-wins (later overrides earlier)
/// - `unsafe_patterns`: accumulated (all are checked)
/// - `join`: accumulated (executed in order)
/// - All other fields: set on `state.options`
pub fn configure(state: &mut State, extension: InputExtension) {
    // First do sub-extensions.
    if let Some(extensions) = extension.extensions {
        for ext in extensions {
            configure(state, ext);
        }
    }

    // Merge handlers (last-wins).
    if let Some(handlers) = extension.handlers {
        state.handlers.extend(handlers);
    }

    // Merge unsafe patterns (accumulate).
    if let Some(unsafe_patterns) = extension.unsafe_patterns {
        state.unsafe_patterns.extend(unsafe_patterns);
    }

    // Merge join functions (accumulate).
    if let Some(join) = extension.join {
        state.join.extend(join);
    }

    // Merge options.
    if let Some(opts) = extension.options {
        apply_options_override(&mut state.options, opts);
    }
}

/// Apply optional overrides to options.
fn apply_options_override(options: &mut Options, overrides: OptionsOverride) {
    if let Some(v) = overrides.bullet {
        options.bullet = v;
    }
    if let Some(v) = overrides.bullet_other {
        options.bullet_other = Some(v);
    }
    if let Some(v) = overrides.bullet_ordered {
        options.bullet_ordered = v;
    }
    if let Some(v) = overrides.close_atx {
        options.close_atx = v;
    }
    if let Some(v) = overrides.emphasis {
        options.emphasis = v;
    }
    if let Some(v) = overrides.fence {
        options.fence = v;
    }
    if let Some(v) = overrides.fences {
        options.fences = v;
    }
    if let Some(v) = overrides.increment_list_marker {
        options.increment_list_marker = v;
    }
    if let Some(v) = overrides.list_item_indent {
        options.list_item_indent = v;
    }
    if let Some(v) = overrides.quote {
        options.quote = v;
    }
    if let Some(v) = overrides.resource_link {
        options.resource_link = v;
    }
    if let Some(v) = overrides.rule {
        options.rule = v;
    }
    if let Some(v) = overrides.rule_repetition {
        options.rule_repetition = v;
    }
    if let Some(v) = overrides.rule_spaces {
        options.rule_spaces = v;
    }
    if let Some(v) = overrides.setext {
        options.setext = v;
    }
    if let Some(v) = overrides.strong {
        options.strong = v;
    }
    if let Some(v) = overrides.tight_definitions {
        options.tight_definitions = v;
    }
}
