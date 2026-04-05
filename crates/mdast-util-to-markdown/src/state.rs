use std::collections::HashMap;

use markdown::mdast::Node;

use crate::types::{
    ConstructName, EncodeSurrounding, HandlerFn, Info, JoinFn, Options, PeekFn, SafeConfig,
    TrackFields, UnsafePattern,
};
use crate::util::container_flow;
use crate::util::container_phrasing;
use crate::util::indent::indent_lines;
use crate::util::safe;
use crate::util::track::Tracker;

/// Core state struct passed around during serialization.
///
/// Mirrors the JS `State` interface from mdast-util-to-markdown.
pub struct State {
    /// Stack of constructs we're currently in.
    pub stack: Vec<ConstructName>,
    /// Positions of child nodes in their parents.
    pub index_stack: Vec<usize>,
    /// Applied handlers, keyed by node type name.
    pub handlers: HashMap<String, HandlerFn>,
    /// Applied peek functions, keyed by node type name.
    pub peek_handlers: HashMap<String, PeekFn>,
    /// Applied unsafe patterns.
    pub unsafe_patterns: Vec<UnsafePattern>,
    /// Applied join handlers.
    pub join: Vec<JoinFn>,
    /// Applied user configuration.
    pub options: Options,
    /// List marker currently in use.
    pub bullet_current: Option<String>,
    /// List marker previously in use.
    pub bullet_last_used: Option<String>,
    /// Info on whether to encode the surrounding of attention (emphasis/strong).
    pub attention_encode_surrounding_info: Option<EncodeSurrounding>,
}

impl State {
    /// Create a new State with default options.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            index_stack: Vec::new(),
            handlers: HashMap::new(),
            peek_handlers: HashMap::new(),
            unsafe_patterns: Vec::new(),
            join: Vec::new(),
            options: Options::default(),
            bullet_current: None,
            bullet_last_used: None,
            attention_encode_surrounding_info: None,
        }
    }

    /// Enter a construct. Push the name onto the stack.
    ///
    /// In JS this returns an `exit` closure. In Rust, use `enter`/`exit` pairs.
    pub fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    /// Exit a construct. Remove the last occurrence of `name` from the stack.
    pub fn exit(&mut self) {
        self.stack.pop();
    }

    /// Call the configured handler for the given node.
    ///
    /// Dispatches based on the node type name, looking it up in `self.handlers`.
    /// Returns an empty string if no handler is registered for the node type.
    pub fn handle(&mut self, node: &Node, parent: Option<&Node>, info: &Info) -> String {
        let type_name = node_type_name(node);
        if let Some(handler) = self.handlers.get(type_name).copied() {
            handler(node, parent, self, info)
        } else {
            String::new()
        }
    }

    /// Peek at the first character a handler would produce for the given node.
    ///
    /// Used by `container_phrasing` to determine the `after` context character.
    pub fn peek(&mut self, node: &Node, parent: Option<&Node>, info: &Info) -> String {
        let type_name = node_type_name(node);
        // First check for a dedicated peek handler
        if let Some(peek_fn) = self.peek_handlers.get(type_name).copied() {
            return peek_fn(node, parent, self, info);
        }
        // Fall back to the regular handler and take the first character
        if let Some(handler) = self.handlers.get(type_name).copied() {
            let result = handler(node, parent, self, info);
            result.chars().next().map_or(String::new(), |c| c.to_string())
        } else {
            String::new()
        }
    }

    /// Make a string safe for embedding in markdown constructs.
    ///
    /// Delegates to `util::safe::safe()`.
    pub fn safe(&self, input: Option<&str>, config: &SafeConfig) -> String {
        safe::safe(self, input, config)
    }

    /// Serialize the children of a parent that contains phrasing children.
    ///
    /// Delegates to `util::container_phrasing::container_phrasing()`.
    pub fn container_phrasing(&mut self, parent: &Node, info: &Info) -> String {
        container_phrasing::container_phrasing(parent, self, info)
    }

    /// Serialize the children of a parent that contains flow children.
    ///
    /// Delegates to `util::container_flow::container_flow()`.
    pub fn container_flow(&mut self, parent: &Node, info: &TrackFields) -> String {
        container_flow::container_flow(parent, self, info)
    }

    /// Create a new position tracker from info.
    pub fn create_tracker(&self, info: &Info) -> Tracker {
        Tracker::new(&TrackFields {
            line: info.line,
            column: info.column,
            line_shift: info.line_shift,
        })
    }

    /// Indent lines of a serialized value using a map function.
    pub fn indent_lines<F>(&self, value: &str, map: F) -> String
    where
        F: Fn(&str, usize, bool) -> String,
    {
        indent_lines(value, map)
    }

    /// Get the association identifier from a node (Definition, LinkReference, ImageReference).
    ///
    /// Prefers `label` over `identifier`. If neither is available, returns empty string.
    pub fn association_id(&self, node: &Node) -> String {
        match node {
            Node::Definition(def) => {
                if let Some(ref label) = def.label {
                    if !label.is_empty() {
                        return label.clone();
                    }
                }
                def.identifier.clone()
            }
            Node::LinkReference(lr) => {
                if let Some(ref label) = lr.label {
                    if !label.is_empty() {
                        return label.clone();
                    }
                }
                lr.identifier.clone()
            }
            Node::ImageReference(ir) => {
                if let Some(ref label) = ir.label {
                    if !label.is_empty() {
                        return label.clone();
                    }
                }
                ir.identifier.clone()
            }
            Node::FootnoteReference(fr) => {
                if let Some(ref label) = fr.label {
                    if !label.is_empty() {
                        return label.clone();
                    }
                }
                fr.identifier.clone()
            }
            _ => String::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the type name string for a node, matching the JS mdast node type strings.
pub fn node_type_name(node: &Node) -> &'static str {
    match node {
        Node::Root(_) => "root",
        Node::Blockquote(_) => "blockquote",
        Node::FootnoteDefinition(_) => "footnoteDefinition",
        Node::List(_) => "list",
        Node::Toml(_) => "toml",
        Node::Yaml(_) => "yaml",
        Node::Break(_) => "break",
        Node::InlineCode(_) => "inlineCode",
        Node::InlineMath(_) => "inlineMath",
        Node::Delete(_) => "delete",
        Node::Emphasis(_) => "emphasis",
        Node::FootnoteReference(_) => "footnoteReference",
        Node::Html(_) => "html",
        Node::Image(_) => "image",
        Node::ImageReference(_) => "imageReference",
        Node::Link(_) => "link",
        Node::LinkReference(_) => "linkReference",
        Node::Strong(_) => "strong",
        Node::Text(_) => "text",
        Node::Code(_) => "code",
        Node::Math(_) => "math",
        Node::Heading(_) => "heading",
        Node::Table(_) => "table",
        Node::ThematicBreak(_) => "thematicBreak",
        Node::TableRow(_) => "tableRow",
        Node::TableCell(_) => "tableCell",
        Node::ListItem(_) => "listItem",
        Node::Definition(_) => "definition",
        Node::Paragraph(_) => "paragraph",
        Node::MdxJsxFlowElement(_) => "mdxJsxFlowElement",
        Node::MdxJsxTextElement(_) => "mdxJsxTextElement",
        Node::MdxFlowExpression(_) => "mdxFlowExpression",
        Node::MdxTextExpression(_) => "mdxTextExpression",
        Node::MdxjsEsm(_) => "mdxjsEsm",
    }
}
