use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;
use crate::util::check::{check_rule, check_rule_repetition};

/// Handle a thematic break node.
///
/// Port of JS `lib/handle/thematic-break.js`.
pub fn handle_thematic_break(
    _node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    _info: &Info,
) -> String {
    let rule = check_rule(&state.options);
    let repetition = check_rule_repetition(&state.options);
    let rule_spaces = state.options.rule_spaces;

    let unit = if rule_spaces {
        format!("{} ", rule)
    } else {
        rule.to_string()
    };

    let value: String = unit.repeat(repetition);

    if rule_spaces {
        // Remove trailing space
        value[..value.len() - 1].to_string()
    } else {
        value
    }
}
