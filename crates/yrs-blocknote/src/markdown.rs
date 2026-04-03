use markdown::mdast::Node;

use crate::blocks::{
    Block, BlockContent, InlineContent, Styles, TableCell, TableCellProps, TableCellType,
    TableContent, TableRow,
};
use crate::schema::BlockType;

// ══════════════════════════════════════════════════════════════
//  Markdown → Blocks  (using `markdown` crate for parsing)
// ══════════════════════════════════════════════════════════════

/// Parse a markdown string into mdast, then convert to Blocks.
pub(crate) fn parse_markdown(md: &str, id_gen: &mut impl FnMut() -> String) -> Vec<Block> {
    let opts = markdown::ParseOptions::gfm();
    let root = markdown::to_mdast(md, &opts).unwrap_or_else(|_| {
        Node::Root(markdown::mdast::Root {
            children: vec![],
            position: None,
        })
    });
    convert_children_to_blocks(&root, id_gen)
}

/// Convert children of a root/container node into blocks.
fn convert_children_to_blocks(node: &Node, id_gen: &mut impl FnMut() -> String) -> Vec<Block> {
    let children = match node {
        Node::Root(r) => &r.children,
        _ => return vec![],
    };
    let mut blocks = Vec::new();
    for child in children {
        if let Node::List(list) = child {
            blocks.extend(expand_list_items(child, list, id_gen));
        } else {
            if is_image_paragraph(child) {
                if let Some(img_block) = extract_image_from_paragraph(child, id_gen) {
                    blocks.push(img_block);
                    continue;
                }
            }
            if let Some(block) = node_to_block(child, id_gen) {
                blocks.push(block);
            }
        }
    }
    blocks
}

/// Expand a List node into individual list item blocks.
fn expand_list_items(
    list_node: &Node,
    list: &markdown::mdast::List,
    id_gen: &mut impl FnMut() -> String,
) -> Vec<Block> {
    let children = match list_node {
        Node::List(l) => &l.children,
        _ => return vec![],
    };
    let mut blocks = Vec::new();
    let start = list.start.unwrap_or(1);

    for item_node in children {
        if let Node::ListItem(item) = item_node {
            if item.checked.is_some() {
                let mut block = task_item_to_block(item_node, item, id_gen);
                if list.spread {
                    block.props.other.insert("listSpread".to_string(), "true".to_string());
                }
                blocks.push(block);
            } else if list.ordered {
                let mut items = list_item_to_block(
                    item_node,
                    BlockType::NumberedListItem,
                    start,
                    id_gen,
                );
                if list.spread {
                    for b in &mut items {
                        b.props.other.insert("listSpread".to_string(), "true".to_string());
                    }
                }
                blocks.extend(items);
            } else {
                let mut items = list_item_to_block(
                    item_node,
                    BlockType::BulletListItem,
                    1,
                    id_gen,
                );
                if list.spread {
                    for b in &mut items {
                        b.props.other.insert("listSpread".to_string(), "true".to_string());
                    }
                }
                blocks.extend(items);
            }
        }
    }
    blocks
}

/// Convert a single mdast node into a Block.
fn node_to_block(node: &Node, id_gen: &mut impl FnMut() -> String) -> Option<Block> {
    match node {
        Node::Heading(h) => {
            let level = h.depth;
            Some(
                Block::new(BlockType::Heading, id_gen())
                    .with_level(level)
                    .with_content(collect_inline_content_from_children(&h.children)),
            )
        }
        Node::Paragraph(p) => Some(
            Block::new(BlockType::Paragraph, id_gen())
                .with_content(collect_inline_content_from_children(&p.children)),
        ),
        Node::Code(cb) => {
            let lang = cb.lang.as_deref().unwrap_or("").to_string();
            let text = cb.value.clone();
            Some(
                Block::new(BlockType::CodeBlock, id_gen())
                    .with_language(lang)
                    .with_content(vec![InlineContent::plain(text)]),
            )
        }
        Node::ThematicBreak(_) => Some(Block::new(BlockType::Divider, id_gen())),
        Node::Table(_) => Some(table_node_to_block(node, id_gen)),
        Node::Blockquote(_) => Some(blockquote_node_to_block(node, id_gen)),
        _ => None,
    }
}

/// Collect list item parts: inline content from first paragraph + child blocks.
fn collect_list_item_parts(
    item: &markdown::mdast::ListItem,
    id_gen: &mut impl FnMut() -> String,
) -> (Vec<InlineContent>, Vec<Block>) {
    let mut content = Vec::new();
    let mut children = Vec::new();

    for child in &item.children {
        match child {
            Node::Paragraph(p) => {
                if content.is_empty() {
                    content = collect_inline_content_from_children(&p.children);
                } else {
                    children.push(
                        Block::new(BlockType::Paragraph, id_gen())
                            .with_content(collect_inline_content_from_children(&p.children)),
                    );
                }
            }
            Node::List(sub_list) => {
                children.extend(expand_list_items(child, sub_list, id_gen));
            }
            _ => {}
        }
    }

    (content, children)
}

fn list_item_to_block(
    node: &Node,
    block_type: BlockType,
    start: u32,
    id_gen: &mut impl FnMut() -> String,
) -> Vec<Block> {
    let Node::ListItem(item) = node else {
        return vec![];
    };
    let (content, children) = collect_list_item_parts(item, id_gen);
    let mut block = Block::new(block_type, id_gen())
        .with_content(content)
        .with_children(children);
    if block_type == BlockType::NumberedListItem && start != 1 {
        block = block.with_start(start as usize);
    }
    vec![block]
}

fn task_item_to_block(
    node: &Node,
    item: &markdown::mdast::ListItem,
    id_gen: &mut impl FnMut() -> String,
) -> Block {
    let Node::ListItem(li) = node else {
        unreachable!()
    };
    let (content, children) = collect_list_item_parts(li, id_gen);
    Block::new(BlockType::CheckListItem, id_gen())
        .with_checked(item.checked.unwrap_or(false))
        .with_content(content)
        .with_children(children)
}

fn table_node_to_block(node: &Node, id_gen: &mut impl FnMut() -> String) -> Block {
    let Node::Table(table) = node else {
        return Block::new(BlockType::Table, id_gen());
    };

    let mut rows = Vec::new();
    for (row_idx, row_node) in table.children.iter().enumerate() {
        if let Node::TableRow(tr) = row_node {
            let is_header = row_idx == 0;
            let mut cells = Vec::new();
            for cell_node in &tr.children {
                if let Node::TableCell(tc) = cell_node {
                    let cell_type = if is_header {
                        TableCellType::TableHeader
                    } else {
                        TableCellType::TableCell
                    };
                    cells.push(TableCell {
                        cell_type,
                        props: TableCellProps::default(),
                        content: collect_inline_content_from_children(&tc.children),
                    });
                }
            }
            rows.push(TableRow { cells });
        }
    }

    let num_columns = rows.first().map_or(0, |r| r.cells.len());
    let column_widths = vec![None; num_columns];

    let header_rows = rows
        .iter()
        .take_while(|row| {
            row.cells
                .first()
                .is_some_and(|c| c.cell_type == TableCellType::TableHeader)
        })
        .count();

    let table_content = TableContent {
        column_widths,
        header_rows: if header_rows > 0 {
            Some(header_rows)
        } else {
            None
        },
        header_cols: None,
        rows,
    };

    Block::new(BlockType::Table, id_gen()).with_table_content(table_content)
}

fn blockquote_node_to_block(node: &Node, id_gen: &mut impl FnMut() -> String) -> Block {
    let Node::Blockquote(bq) = node else {
        return Block::new(BlockType::Quote, id_gen());
    };

    let mut inline_content = Vec::new();
    let mut children = Vec::new();
    let mut first_para = true;

    for child in &bq.children {
        match child {
            Node::Paragraph(p) if first_para => {
                inline_content = collect_inline_content_from_children(&p.children);
                first_para = false;
            }
            _ => {
                if let Some(block) = node_to_block(child, id_gen) {
                    children.push(block);
                }
            }
        }
    }

    Block::new(BlockType::Quote, id_gen())
        .with_content(inline_content)
        .with_children(children)
}

// ── Inline content extraction ─────────────────────────────────

fn collect_inline_content_from_children(children: &[Node]) -> Vec<InlineContent> {
    let mut result = Vec::new();
    collect_inlines_recursive(children, &Styles::default(), &mut result);
    result
}

fn collect_inlines_recursive(
    children: &[Node],
    parent_styles: &Styles,
    result: &mut Vec<InlineContent>,
) {
    for child in children {
        match child {
            Node::Text(t) => {
                result.push(InlineContent::styled(t.value.clone(), parent_styles.clone()));
            }
            Node::InlineCode(c) => {
                result.push(InlineContent::styled(
                    c.value.clone(),
                    parent_styles.clone().with_code(),
                ));
            }
            Node::Break(_) => {
                result.push(InlineContent::HardBreak);
            }
            Node::Strong(s) => {
                collect_inlines_recursive(
                    &s.children,
                    &parent_styles.clone().with_bold(),
                    result,
                );
            }
            Node::Emphasis(e) => {
                collect_inlines_recursive(
                    &e.children,
                    &parent_styles.clone().with_italic(),
                    result,
                );
            }
            Node::Delete(d) => {
                collect_inlines_recursive(
                    &d.children,
                    &parent_styles.clone().with_strikethrough(),
                    result,
                );
            }
            Node::Link(link) => {
                let mut link_content = Vec::new();
                collect_inlines_recursive(&link.children, parent_styles, &mut link_content);
                result.push(InlineContent::Link {
                    href: link.url.clone(),
                    content: link_content,
                });
            }
            // SoftBreak: treat as space
            Node::Image(img) => {
                // Image inline in a paragraph — treat as text for the alt
                result.push(InlineContent::styled(
                    img.alt.clone(),
                    parent_styles.clone(),
                ));
            }
            _ => {
                // For unknown inline nodes, try to recurse into their children
                if let Some(children) = get_node_children(child) {
                    collect_inlines_recursive(children, parent_styles, result);
                }
            }
        }
    }
}

fn get_node_children(node: &Node) -> Option<&Vec<Node>> {
    match node {
        Node::Paragraph(p) => Some(&p.children),
        Node::Emphasis(e) => Some(&e.children),
        Node::Strong(s) => Some(&s.children),
        Node::Delete(d) => Some(&d.children),
        Node::Link(l) => Some(&l.children),
        _ => None,
    }
}

fn is_image_paragraph(node: &Node) -> bool {
    let Node::Paragraph(p) = node else {
        return false;
    };
    if p.children.len() != 1 {
        return false;
    }
    matches!(&p.children[0], Node::Image(_))
}

fn extract_image_from_paragraph(
    node: &Node,
    id_gen: &mut impl FnMut() -> String,
) -> Option<Block> {
    let Node::Paragraph(p) = node else {
        return None;
    };
    for child in &p.children {
        if let Node::Image(img) = child {
            return Some(
                Block::new(BlockType::Image, id_gen())
                    .with_url(img.url.clone())
                    .with_caption(img.alt.clone()),
            );
        }
    }
    None
}

// ══════════════════════════════════════════════════════════════
//  Blocks → Markdown  (using mdast-util-to-markdown for rendering)
// ══════════════════════════════════════════════════════════════

#[allow(clippy::unnecessary_wraps)] // public API returns Result for forward-compat
pub(crate) fn blocks_to_markdown(blocks: &[Block]) -> crate::ConvertResult<String> {
    let root = blocks_to_mdast(blocks);
    let opts = mdast_util_to_markdown::Options {
        bullet: '-',
        ..mdast_util_to_markdown::Options::default()
    };
    let result = mdast_util_to_markdown::to_markdown(&root, &opts);
    Ok(result)
}

/// Convert a slice of Blocks into an mdast Root node.
fn blocks_to_mdast(blocks: &[Block]) -> Node {
    let children = blocks_to_mdast_children(blocks);
    Node::Root(markdown::mdast::Root {
        children,
        position: None,
    })
}

/// Convert blocks to mdast children, grouping consecutive list items.
fn blocks_to_mdast_children(blocks: &[Block]) -> Vec<Node> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < blocks.len() {
        match blocks[i].block_type {
            BlockType::BulletListItem => {
                let (list_node, consumed) = group_bullet_list(&blocks[i..]);
                result.push(list_node);
                i += consumed;
            }
            BlockType::NumberedListItem => {
                let (list_node, consumed) = group_numbered_list(&blocks[i..]);
                result.push(list_node);
                i += consumed;
            }
            BlockType::CheckListItem => {
                let (list_node, consumed) = group_check_list(&blocks[i..]);
                result.push(list_node);
                i += consumed;
            }
            _ => {
                result.push(block_to_mdast(&blocks[i]));
                i += 1;
            }
        }
    }

    result
}

/// Group consecutive `BulletListItem` blocks into a single List node.
fn group_bullet_list(blocks: &[Block]) -> (Node, usize) {
    let mut items = Vec::new();
    let mut count = 0;
    let mut spread = false;

    for block in blocks {
        if block.block_type != BlockType::BulletListItem {
            break;
        }
        if block.props.other.get("listSpread").is_some_and(|v| v == "true") {
            spread = true;
        }
        items.push(block_to_list_item(block));
        count += 1;
    }

    let list = Node::List(markdown::mdast::List {
        ordered: false,
        start: None,
        spread,
        children: items,
        position: None,
    });
    (list, count)
}

/// Group consecutive `NumberedListItem` blocks into a single List node.
fn group_numbered_list(blocks: &[Block]) -> (Node, usize) {
    let mut items = Vec::new();
    let mut count = 0;
    let mut spread = false;
    let start = blocks
        .first()
        .and_then(|b| b.props.start)
        .unwrap_or(1);

    for block in blocks {
        if block.block_type != BlockType::NumberedListItem {
            break;
        }
        if block.props.other.get("listSpread").is_some_and(|v| v == "true") {
            spread = true;
        }
        items.push(block_to_list_item(block));
        count += 1;
    }

    let list = Node::List(markdown::mdast::List {
        ordered: true,
        start: Some(start as u32),
        spread,
        children: items,
        position: None,
    });
    (list, count)
}

/// Group consecutive `CheckListItem` blocks into a single List node.
fn group_check_list(blocks: &[Block]) -> (Node, usize) {
    let mut items = Vec::new();
    let mut count = 0;
    let mut spread = false;

    for block in blocks {
        if block.block_type != BlockType::CheckListItem {
            break;
        }
        if block.props.other.get("listSpread").is_some_and(|v| v == "true") {
            spread = true;
        }
        let checked = block.props.checked.unwrap_or(false);
        items.push(block_to_check_list_item(block, checked));
        count += 1;
    }

    let list = Node::List(markdown::mdast::List {
        ordered: false,
        start: None,
        spread,
        children: items,
        position: None,
    });
    (list, count)
}

/// Convert a single Block to a `ListItem` node.
fn block_to_list_item(block: &Block) -> Node {
    let inline_content = block.content.as_inline();
    let mut children: Vec<Node> = Vec::new();

    // First paragraph from inline content
    if !inline_content.is_empty() {
        children.push(Node::Paragraph(markdown::mdast::Paragraph {
            children: inline_to_mdast(inline_content),
            position: None,
        }));
    }

    // Nested children become sub-nodes
    if !block.children.is_empty() {
        let child_nodes = blocks_to_mdast_children(&block.children);
        children.extend(child_nodes);
    }

    Node::ListItem(markdown::mdast::ListItem {
        checked: None,
        spread: false,
        children,
        position: None,
    })
}

/// Convert a check list Block to a `ListItem` node with checked status.
fn block_to_check_list_item(block: &Block, checked: bool) -> Node {
    let inline_content = block.content.as_inline();
    let mut children: Vec<Node> = Vec::new();

    if !inline_content.is_empty() {
        children.push(Node::Paragraph(markdown::mdast::Paragraph {
            children: inline_to_mdast(inline_content),
            position: None,
        }));
    }

    if !block.children.is_empty() {
        let child_nodes = blocks_to_mdast_children(&block.children);
        children.extend(child_nodes);
    }

    Node::ListItem(markdown::mdast::ListItem {
        checked: Some(checked),
        spread: false,
        children,
        position: None,
    })
}

/// Convert a single Block to an mdast Node.
fn block_to_mdast(block: &Block) -> Node {
    let inline_content = block.content.as_inline();

    match block.block_type {
        BlockType::Heading => {
            let level = block.props.level.unwrap_or(1);
            Node::Heading(markdown::mdast::Heading {
                depth: level,
                children: inline_to_mdast(inline_content),
                position: None,
            })
        }
        BlockType::Paragraph | BlockType::HardBreak => {
            Node::Paragraph(markdown::mdast::Paragraph {
                children: inline_to_mdast(inline_content),
                position: None,
            })
        }
        BlockType::CodeBlock => {
            let lang = block.props.language.clone().unwrap_or_default();
            let value = inline_content
                .first()
                .map_or_else(String::new, |c| match c {
                    InlineContent::Text { text, .. } => text.clone(),
                    _ => String::new(),
                });
            Node::Code(markdown::mdast::Code {
                lang: if lang.is_empty() { None } else { Some(lang) },
                meta: None,
                value,
                position: None,
            })
        }
        BlockType::Image => {
            let url = block.props.url.as_deref().unwrap_or_default().to_string();
            let alt = block.props.caption.as_deref().unwrap_or_default().to_string();
            Node::Paragraph(markdown::mdast::Paragraph {
                children: vec![Node::Image(markdown::mdast::Image {
                    url,
                    alt,
                    title: None,
                    position: None,
                })],
                position: None,
            })
        }
        BlockType::Divider => Node::ThematicBreak(markdown::mdast::ThematicBreak { position: None }),
        BlockType::Table => table_block_to_mdast(block),
        BlockType::Quote => {
            let mut bq_children = Vec::new();
            // First paragraph from inline content
            if !inline_content.is_empty() {
                bq_children.push(Node::Paragraph(markdown::mdast::Paragraph {
                    children: inline_to_mdast(inline_content),
                    position: None,
                }));
            }
            // Children blocks
            for child in &block.children {
                bq_children.push(block_to_mdast(child));
            }
            Node::Blockquote(markdown::mdast::Blockquote {
                children: bq_children,
                position: None,
            })
        }
        BlockType::ToggleListItem => {
            // Degrade to bullet list item
            let list_item = block_to_list_item(block);
            Node::List(markdown::mdast::List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![list_item],
                position: None,
            })
        }
        // BulletListItem, NumberedListItem, CheckListItem should be grouped
        // but if they appear standalone, wrap them
        BlockType::BulletListItem => {
            let list_item = block_to_list_item(block);
            Node::List(markdown::mdast::List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![list_item],
                position: None,
            })
        }
        BlockType::NumberedListItem => {
            let start = block.props.start.unwrap_or(1) as u32;
            let list_item = block_to_list_item(block);
            Node::List(markdown::mdast::List {
                ordered: true,
                start: Some(start),
                spread: false,
                children: vec![list_item],
                position: None,
            })
        }
        BlockType::CheckListItem => {
            let checked = block.props.checked.unwrap_or(false);
            let list_item = block_to_check_list_item(block, checked);
            Node::List(markdown::mdast::List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![list_item],
                position: None,
            })
        }
        // Table sub-types and other structural types → paragraph
        BlockType::TableRow
        | BlockType::TableHeader
        | BlockType::TableCell
        | BlockType::TableParagraph => Node::Paragraph(markdown::mdast::Paragraph {
            children: inline_to_mdast(inline_content),
            position: None,
        }),
    }
}

fn table_block_to_mdast(block: &Block) -> Node {
    let BlockContent::Table(table_content) = &block.content else {
        return Node::Paragraph(markdown::mdast::Paragraph {
            children: vec![],
            position: None,
        });
    };

    let num_columns = table_content.rows.first().map_or(0, |r| r.cells.len());
    let align = vec![markdown::mdast::AlignKind::None; num_columns];

    let mut mdast_rows = Vec::new();
    for table_row in &table_content.rows {
        let mut mdast_cells = Vec::new();
        for cell in &table_row.cells {
            mdast_cells.push(Node::TableCell(markdown::mdast::TableCell {
                children: inline_to_mdast(&cell.content),
                position: None,
            }));
        }
        mdast_rows.push(Node::TableRow(markdown::mdast::TableRow {
            children: mdast_cells,
            position: None,
        }));
    }

    Node::Table(markdown::mdast::Table {
        align,
        children: mdast_rows,
        position: None,
    })
}

// ── Inline content → mdast nodes ─────────────────────────────

fn inline_to_mdast(content: &[InlineContent]) -> Vec<Node> {
    let mut result = Vec::new();
    for item in content {
        match item {
            InlineContent::Text { text, styles } => {
                let inner = if styles.code {
                    Node::InlineCode(markdown::mdast::InlineCode {
                        value: text.clone(),
                        position: None,
                    })
                } else {
                    Node::Text(markdown::mdast::Text {
                        value: text.clone(),
                        position: None,
                    })
                };
                result.push(wrap_with_styles(inner, styles));
            }
            InlineContent::Link { href, content } => {
                let link_children = inline_to_mdast(content);
                result.push(Node::Link(markdown::mdast::Link {
                    url: href.clone(),
                    title: None,
                    children: link_children,
                    position: None,
                }));
            }
            InlineContent::HardBreak => {
                result.push(Node::Break(markdown::mdast::Break { position: None }));
            }
        }
    }
    result
}

/// Wrap a node with style wrappers (bold, italic, strikethrough).
/// Code is already handled at the leaf level.
fn wrap_with_styles(inner: Node, styles: &Styles) -> Node {
    let mut current = inner;
    // Apply strikethrough first (innermost wrapping after text)
    if styles.strikethrough {
        current = Node::Delete(markdown::mdast::Delete {
            children: vec![current],
            position: None,
        });
    }
    if styles.italic {
        current = Node::Emphasis(markdown::mdast::Emphasis {
            children: vec![current],
            position: None,
        });
    }
    if styles.bold {
        current = Node::Strong(markdown::mdast::Strong {
            children: vec![current],
            position: None,
        });
    }
    current
}
