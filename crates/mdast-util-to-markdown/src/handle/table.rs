use markdown::mdast::{AlignKind, Node};

use crate::state::State;
use crate::types::{ConstructName, Info};

/// Handle a table node.
///
/// Port of JS `mdast-util-gfm-table` toMarkdown handler.
pub fn handle_table(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Table(table) = node {
        let mut matrix: Vec<Vec<String>> = Vec::new();
        state.enter(ConstructName::Table);

        for child in &table.children {
            matrix.push(handle_table_row_as_data(child, state, info));
        }

        state.exit(); // table

        serialize_data(&matrix, Some(&table.align), true, true)
    } else {
        String::new()
    }
}

/// Handle a table row node (standalone).
pub fn handle_table_row(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    let row = handle_table_row_as_data(node, state, info);
    let value = serialize_data(&[row], None, true, true);
    // markdown-table always adds an align row - only return the first line
    if let Some(idx) = value.find('\n') {
        value[..idx].to_string()
    } else {
        value
    }
}

/// Handle a table cell node (standalone).
pub fn handle_table_cell(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    handle_table_cell_value(node, state, info, true)
}

/// Serialize a table cell to its string value.
fn handle_table_cell_value(
    node: &Node,
    state: &mut State,
    info: &Info,
    padding: bool,
) -> String {
    let around = if padding { " " } else { "|" };
    state.enter(ConstructName::TableCell);
    state.enter(ConstructName::Phrasing);
    let value = state.container_phrasing(node, &Info {
        before: around.to_string(),
        after: around.to_string(),
        line: info.line,
        column: info.column,
        line_shift: info.line_shift,
    });
    state.exit(); // phrasing
    state.exit(); // tableCell
    value
}

/// Convert a table row node to a vector of cell strings.
fn handle_table_row_as_data(node: &Node, state: &mut State, info: &Info) -> Vec<String> {
    let children = match node {
        Node::TableRow(row) => &row.children,
        _ => return vec![],
    };

    state.enter(ConstructName::TableRow);
    let mut result = Vec::new();
    for child in children {
        result.push(handle_table_cell_value(child, state, info, true));
    }
    state.exit(); // tableRow
    result
}

/// Format a matrix of strings into a markdown table.
///
/// Simplified port of the `markdown-table` npm package.
fn serialize_data(
    matrix: &[Vec<String>],
    align: Option<&Vec<AlignKind>>,
    align_delimiters: bool,
    padding: bool,
) -> String {
    // Determine number of columns
    let column_count = matrix.iter().map(|row| row.len()).max().unwrap_or(0);
    if column_count == 0 {
        return String::new();
    }

    // Get alignment for each column
    let alignments: Vec<AlignKind> = (0..column_count)
        .map(|i| {
            align
                .and_then(|a| a.get(i).cloned())
                .unwrap_or(AlignKind::None)
        })
        .collect();

    // Calculate column widths
    let mut widths: Vec<usize> = vec![3; column_count]; // minimum width for delimiter row

    if align_delimiters {
        for row in matrix {
            for (i, cell) in row.iter().enumerate() {
                let len = string_length(cell);
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();

    for (row_idx, row) in matrix.iter().enumerate() {
        let mut cells: Vec<String> = Vec::new();

        for col in 0..column_count {
            let cell = row.get(col).map(|s| s.as_str()).unwrap_or("");

            if align_delimiters {
                let cell_len = string_length(cell);
                let pad_len = widths[col].saturating_sub(cell_len);

                match alignments[col] {
                    AlignKind::Right => {
                        cells.push(format!("{}{}", " ".repeat(pad_len), cell));
                    }
                    AlignKind::Center => {
                        let left = pad_len / 2;
                        let right = pad_len - left;
                        cells.push(format!(
                            "{}{}{}",
                            " ".repeat(left),
                            cell,
                            " ".repeat(right)
                        ));
                    }
                    _ => {
                        cells.push(format!("{}{}", cell, " ".repeat(pad_len)));
                    }
                }
            } else {
                cells.push(cell.to_string());
            }
        }

        // Build line
        let line = if padding {
            format!("| {} |", cells.join(" | "))
        } else {
            format!("|{}|", cells.join("|"))
        };
        lines.push(line);

        // After the first row, insert the delimiter row
        if row_idx == 0 {
            let mut delimiters: Vec<String> = Vec::new();
            for col in 0..column_count {
                let width = if align_delimiters {
                    widths[col]
                } else {
                    3
                };
                let dash_count = width.max(1);
                match alignments[col] {
                    AlignKind::Left => {
                        delimiters
                            .push(format!(":{}", "-".repeat(dash_count.saturating_sub(1))));
                    }
                    AlignKind::Right => {
                        delimiters
                            .push(format!("{}:", "-".repeat(dash_count.saturating_sub(1))));
                    }
                    AlignKind::Center => {
                        delimiters.push(format!(
                            ":{}:",
                            "-".repeat(dash_count.saturating_sub(2).max(1))
                        ));
                    }
                    AlignKind::None => {
                        delimiters.push("-".repeat(dash_count));
                    }
                }
            }
            let delimiter_line = if padding {
                format!("| {} |", delimiters.join(" | "))
            } else {
                format!("|{}|", delimiters.join("|"))
            };
            lines.push(delimiter_line);
        }
    }

    lines.join("\n")
}

/// Get the visual length of a string (simple: just char count).
fn string_length(value: &str) -> usize {
    value.chars().count()
}
