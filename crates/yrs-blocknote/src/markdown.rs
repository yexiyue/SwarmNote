use comrak::nodes::{
    ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList, NodeTaskItem, NodeValue,
    Sourcepos, TableAlignment,
};
use comrak::{Arena, Options, format_commonmark};

use crate::blocks::{
    Block, BlockContent, InlineContent, Styles, TableCell, TableCellProps, TableCellType,
    TableContent, TableRow,
};
use crate::schema::BlockType;

// ── Shared comrak options ─────────────────────────────────────

pub(crate) fn comrak_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts
}

// ── Markdown → Blocks ─────────────────────────────────────────

pub(crate) fn convert_children_to_blocks_toplevel<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in node.children() {
        let data = child.data.borrow();
        if let NodeValue::List(list) = &data.value {
            let list_type = list.list_type;
            let start = list.start;
            drop(data);
            blocks.extend(expand_list_items(child, list_type, start, id_gen));
        } else {
            drop(data);
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

fn expand_list_items<'a>(
    list_node: &'a comrak::nodes::AstNode<'a>,
    list_type: ListType,
    start: usize,
    id_gen: &mut impl FnMut() -> String,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for item_node in list_node.children() {
        let item_data = item_node.data.borrow();
        match &item_data.value {
            NodeValue::Item(_) => {
                drop(item_data);
                blocks.extend(list_item_to_block(item_node, list_type, start, id_gen));
            }
            NodeValue::TaskItem(task) => {
                let task = *task;
                drop(item_data);
                blocks.push(task_item_to_block(item_node, &task, id_gen));
            }
            _ => {
                drop(item_data);
            }
        }
    }
    blocks
}

fn node_to_block<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> Option<Block> {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Heading(h) => {
            let level = h.level;
            drop(data);
            Some(
                Block::new(BlockType::Heading, id_gen())
                    .with_level(level)
                    .with_content(collect_inline_content(node)),
            )
        }
        NodeValue::Paragraph => {
            drop(data);
            Some(
                Block::new(BlockType::Paragraph, id_gen())
                    .with_content(collect_inline_content(node)),
            )
        }
        NodeValue::CodeBlock(cb) => {
            let lang = cb.info.split_whitespace().next().unwrap_or("").to_string();
            let text = cb.literal.trim_end_matches('\n').to_string();
            drop(data);
            Some(
                Block::new(BlockType::CodeBlock, id_gen())
                    .with_language(lang)
                    .with_content(vec![InlineContent::plain(text)]),
            )
        }
        NodeValue::ThematicBreak => {
            drop(data);
            Some(Block::new(BlockType::Divider, id_gen()))
        }
        NodeValue::Table(_) => {
            drop(data);
            Some(table_node_to_block(node, id_gen))
        }
        NodeValue::BlockQuote => {
            drop(data);
            Some(blockquote_node_to_block(node, id_gen))
        }
        _ => None,
    }
}

fn collect_list_item_parts<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> (Vec<InlineContent>, Vec<Block>) {
    let mut content = Vec::new();
    let mut children = Vec::new();

    for child in node.children() {
        let child_data = child.data.borrow();
        match &child_data.value {
            NodeValue::Paragraph => {
                drop(child_data);
                if content.is_empty() {
                    content = collect_inline_content(child);
                } else {
                    children.push(
                        Block::new(BlockType::Paragraph, id_gen())
                            .with_content(collect_inline_content(child)),
                    );
                }
            }
            NodeValue::List(sub_list) => {
                let sub_type = sub_list.list_type;
                let sub_start = sub_list.start;
                drop(child_data);
                children.extend(expand_list_items(child, sub_type, sub_start, id_gen));
            }
            _ => {
                drop(child_data);
            }
        }
    }

    (content, children)
}

fn list_item_to_block<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    list_type: ListType,
    start: usize,
    id_gen: &mut impl FnMut() -> String,
) -> Vec<Block> {
    let block_type = match list_type {
        ListType::Bullet => BlockType::BulletListItem,
        ListType::Ordered => BlockType::NumberedListItem,
    };
    let (content, children) = collect_list_item_parts(node, id_gen);
    let mut block = Block::new(block_type, id_gen())
        .with_content(content)
        .with_children(children);
    if list_type == ListType::Ordered && start != 1 {
        block = block.with_start(start);
    }
    vec![block]
}

fn task_item_to_block<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    task: &NodeTaskItem,
    id_gen: &mut impl FnMut() -> String,
) -> Block {
    let (content, children) = collect_list_item_parts(node, id_gen);
    Block::new(BlockType::CheckListItem, id_gen())
        .with_checked(task.symbol.is_some())
        .with_content(content)
        .with_children(children)
}

fn table_node_to_block<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> Block {
    let mut rows = Vec::new();

    for row_node in node.children() {
        let row_data = row_node.data.borrow();
        if let NodeValue::TableRow(is_header) = &row_data.value {
            let is_header = *is_header;
            drop(row_data);
            let mut cells = Vec::new();
            for cell_node in row_node.children() {
                let cell_data = cell_node.data.borrow();
                if matches!(&cell_data.value, NodeValue::TableCell) {
                    let cell_type = if is_header {
                        TableCellType::TableHeader
                    } else {
                        TableCellType::TableCell
                    };
                    drop(cell_data);
                    cells.push(TableCell {
                        cell_type,
                        props: TableCellProps::default(),
                        content: collect_inline_content(cell_node),
                    });
                }
            }
            rows.push(TableRow { cells });
        }
    }

    // Determine column count from first row
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

fn blockquote_node_to_block<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> Block {
    // A blockquote may contain multiple children (paragraphs, lists, etc.)
    // We take the first paragraph's inline content as the Quote block content,
    // and any remaining children become Block children.
    let mut inline_content = Vec::new();
    let mut children = Vec::new();
    let mut first_para = true;

    for child in node.children() {
        let child_data = child.data.borrow();
        match &child_data.value {
            NodeValue::Paragraph if first_para => {
                drop(child_data);
                inline_content = collect_inline_content(child);
                first_para = false;
            }
            _ => {
                drop(child_data);
                // Convert remaining children as nested blocks
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

fn collect_inline_content<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Vec<InlineContent> {
    let mut result = Vec::new();
    collect_inlines_recursive(node, &Styles::default(), &mut result);
    result
}

fn collect_inlines_recursive<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    parent_styles: &Styles,
    result: &mut Vec<InlineContent>,
) {
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(text) => {
                let text = text.to_string();
                drop(data);
                result.push(InlineContent::styled(text, parent_styles.clone()));
            }
            NodeValue::Code(NodeCode { literal, .. }) => {
                let text = literal.clone();
                drop(data);
                result.push(InlineContent::styled(
                    text,
                    parent_styles.clone().with_code(),
                ));
            }
            NodeValue::SoftBreak => {
                drop(data);
                result.push(InlineContent::styled(" ", parent_styles.clone()));
            }
            NodeValue::LineBreak => {
                drop(data);
                result.push(InlineContent::HardBreak);
            }
            NodeValue::Strong => {
                drop(data);
                collect_inlines_recursive(child, &parent_styles.clone().with_bold(), result);
            }
            NodeValue::Emph => {
                drop(data);
                collect_inlines_recursive(child, &parent_styles.clone().with_italic(), result);
            }
            NodeValue::Strikethrough => {
                drop(data);
                collect_inlines_recursive(
                    child,
                    &parent_styles.clone().with_strikethrough(),
                    result,
                );
            }
            NodeValue::Link(link) => {
                let url = link.url.clone();
                drop(data);
                // Collect link's inner content with parent styles (no link in styles anymore)
                let mut link_content = Vec::new();
                collect_inlines_recursive(child, parent_styles, &mut link_content);
                result.push(InlineContent::Link {
                    href: url,
                    content: link_content,
                });
            }
            _ => {
                drop(data);
                collect_inlines_recursive(child, parent_styles, result);
            }
        }
    }
}

fn is_image_paragraph<'a>(node: &'a comrak::nodes::AstNode<'a>) -> bool {
    let data = node.data.borrow();
    if !matches!(&data.value, NodeValue::Paragraph) {
        return false;
    }
    drop(data);
    let mut children = node.children();
    let first = children.next();
    let second = children.next();
    if second.is_some() {
        return false;
    }
    if let Some(first) = first {
        let first_data = first.data.borrow();
        matches!(&first_data.value, NodeValue::Image(_))
    } else {
        false
    }
}

fn extract_image_from_paragraph<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    id_gen: &mut impl FnMut() -> String,
) -> Option<Block> {
    for child in node.children() {
        let data = child.data.borrow();
        if let NodeValue::Image(img) = &data.value {
            let url = img.url.clone();
            drop(data);
            let mut caption = String::new();
            for alt_child in child.children() {
                let alt_data = alt_child.data.borrow();
                if let NodeValue::Text(t) = &alt_data.value {
                    caption = t.to_string();
                }
            }
            return Some(
                Block::new(BlockType::Image, id_gen())
                    .with_url(url)
                    .with_caption(caption),
            );
        }
    }
    None
}

// ── Blocks → Markdown ─────────────────────────────────────────

pub(crate) fn blocks_to_markdown(blocks: &[Block]) -> crate::ConvertResult<String> {
    let arena = Arena::new();
    let opts = comrak_options();
    let doc = arena.alloc(NodeValue::Document.into());

    for block in blocks {
        build_comrak_node(block, doc, &arena);
    }

    let mut output = String::new();
    format_commonmark(doc, &opts, &mut output)?;
    Ok(output)
}

fn build_comrak_node<'a>(
    block: &Block,
    parent: &'a comrak::nodes::AstNode<'a>,
    arena: &'a Arena<'a>,
) {
    let inline_content = block.content.as_inline();

    match block.block_type {
        BlockType::Heading => {
            let level = block.props.level.unwrap_or(1);
            let h = arena.alloc(
                NodeValue::Heading(NodeHeading {
                    level,
                    setext: false,
                    closed: false,
                })
                .into(),
            );
            append_inline_content(inline_content, h, arena);
            parent.append(h);
        }
        BlockType::BulletListItem => {
            build_list_item_node(block, parent, arena, ListType::Bullet, 1);
        }
        BlockType::NumberedListItem => {
            let start = block.props.start.unwrap_or(1);
            build_list_item_node(block, parent, arena, ListType::Ordered, start);
        }
        BlockType::CheckListItem => {
            let checked = block.props.checked.unwrap_or(false);
            let list = arena.alloc(
                NodeValue::List(NodeList {
                    list_type: ListType::Bullet,
                    start: 1,
                    delimiter: comrak::nodes::ListDelimType::Period,
                    bullet_char: b'-',
                    tight: true,
                    marker_offset: 0,
                    padding: 2,
                    is_task_list: true,
                })
                .into(),
            );
            let item = arena.alloc(
                NodeValue::TaskItem(NodeTaskItem {
                    symbol: if checked { Some('x') } else { None },
                    symbol_sourcepos: Sourcepos {
                        start: comrak::nodes::LineColumn { line: 0, column: 0 },
                        end: comrak::nodes::LineColumn { line: 0, column: 0 },
                    },
                })
                .into(),
            );
            let p = arena.alloc(NodeValue::Paragraph.into());
            append_inline_content(inline_content, p, arena);
            item.append(p);
            list.append(item);
            parent.append(list);
        }
        BlockType::CodeBlock => {
            let lang = block
                .props
                .language
                .as_deref()
                .unwrap_or_default()
                .to_string();
            let literal = inline_content.first().map_or_else(
                || "\n".into(),
                |c| match c {
                    InlineContent::Text { text, .. } => {
                        let mut s = text.clone();
                        if !s.ends_with('\n') {
                            s.push('\n');
                        }
                        s
                    }
                    _ => "\n".into(),
                },
            );
            let cb = arena.alloc(
                NodeValue::CodeBlock(Box::new(NodeCodeBlock {
                    fenced: true,
                    fence_char: b'`',
                    fence_length: 3,
                    fence_offset: 0,
                    info: lang,
                    literal,
                    closed: true,
                }))
                .into(),
            );
            parent.append(cb);
        }
        BlockType::Image => {
            let url = block.props.url.as_deref().unwrap_or_default().to_string();
            let caption = block
                .props
                .caption
                .as_deref()
                .unwrap_or_default()
                .to_string();
            let p = arena.alloc(NodeValue::Paragraph.into());
            let img = arena.alloc(
                NodeValue::Image(Box::new(NodeLink {
                    url,
                    title: String::new(),
                }))
                .into(),
            );
            if !caption.is_empty() {
                let alt = arena.alloc(NodeValue::Text(caption.into()).into());
                img.append(alt);
            }
            p.append(img);
            parent.append(p);
        }
        BlockType::Divider => {
            let tb = arena.alloc(NodeValue::ThematicBreak.into());
            parent.append(tb);
        }
        BlockType::Table => {
            build_table_node(block, parent, arena);
        }
        BlockType::Quote => {
            let bq = arena.alloc(NodeValue::BlockQuote.into());
            let p = arena.alloc(NodeValue::Paragraph.into());
            append_inline_content(inline_content, p, arena);
            bq.append(p);
            // Append children as nested blocks inside the blockquote
            for child in &block.children {
                build_comrak_node(child, bq, arena);
            }
            parent.append(bq);
        }
        BlockType::ToggleListItem => {
            // Degrade to bullet list item in Markdown
            build_list_item_node(block, parent, arena, ListType::Bullet, 1);
        }
        // Paragraph + structural/unknown types render as paragraph
        BlockType::Paragraph
        | BlockType::TableRow
        | BlockType::TableHeader
        | BlockType::TableCell
        | BlockType::TableParagraph
        | BlockType::HardBreak => {
            let p = arena.alloc(NodeValue::Paragraph.into());
            append_inline_content(inline_content, p, arena);
            parent.append(p);
        }
    }
}

fn build_list_item_node<'a>(
    block: &Block,
    parent: &'a comrak::nodes::AstNode<'a>,
    arena: &'a Arena<'a>,
    list_type: ListType,
    start: usize,
) {
    let inline_content = block.content.as_inline();
    let (bullet_char, padding) = match list_type {
        ListType::Bullet => (b'-', 2),
        ListType::Ordered => (b'.', 3),
    };
    let nl = NodeList {
        list_type,
        start,
        delimiter: comrak::nodes::ListDelimType::Period,
        bullet_char,
        tight: true,
        marker_offset: 0,
        padding,
        is_task_list: false,
    };
    let list = arena.alloc(NodeValue::List(nl).into());
    let item = arena.alloc(
        NodeValue::Item(NodeList {
            list_type,
            start,
            delimiter: comrak::nodes::ListDelimType::Period,
            bullet_char,
            tight: true,
            marker_offset: 0,
            padding,
            is_task_list: false,
        })
        .into(),
    );
    let p = arena.alloc(NodeValue::Paragraph.into());
    append_inline_content(inline_content, p, arena);
    item.append(p);
    for child in &block.children {
        build_comrak_node(child, item, arena);
    }
    list.append(item);
    parent.append(list);
}

fn append_inline_content<'a>(
    content: &[InlineContent],
    parent: &'a comrak::nodes::AstNode<'a>,
    arena: &'a Arena<'a>,
) {
    for item in content {
        match item {
            InlineContent::Text { text, styles } => {
                if styles.code {
                    let code_node = arena.alloc(
                        NodeValue::Code(NodeCode {
                            num_backticks: 1,
                            literal: text.clone(),
                        })
                        .into(),
                    );
                    wrap_with_marks(code_node, styles, parent, arena);
                } else {
                    let text_node = arena.alloc(NodeValue::Text(text.clone().into()).into());
                    wrap_with_marks(text_node, styles, parent, arena);
                }
            }
            InlineContent::Link { href, content } => {
                let link_node = arena.alloc(
                    NodeValue::Link(Box::new(NodeLink {
                        url: href.clone(),
                        title: String::new(),
                    }))
                    .into(),
                );
                // Append link's inner content
                append_inline_content(content, link_node, arena);
                parent.append(link_node);
            }
            InlineContent::HardBreak => {
                let lb = arena.alloc(NodeValue::LineBreak.into());
                parent.append(lb);
            }
        }
    }
}

fn wrap_with_marks<'a>(
    inner: &'a comrak::nodes::AstNode<'a>,
    styles: &Styles,
    parent: &'a comrak::nodes::AstNode<'a>,
    arena: &'a Arena<'a>,
) {
    let mut current = inner;
    if styles.strikethrough {
        let w = arena.alloc(NodeValue::Strikethrough.into());
        w.append(current);
        current = w;
    }
    if styles.italic {
        let w = arena.alloc(NodeValue::Emph.into());
        w.append(current);
        current = w;
    }
    if styles.bold {
        let w = arena.alloc(NodeValue::Strong.into());
        w.append(current);
        current = w;
    }
    // Link is now handled as InlineContent::Link, not a style
    parent.append(current);
}

fn build_table_node<'a>(
    block: &Block,
    parent: &'a comrak::nodes::AstNode<'a>,
    arena: &'a Arena<'a>,
) {
    let BlockContent::Table(table_content) = &block.content else {
        return;
    };

    let num_columns = table_content
        .rows
        .first()
        .map_or(0, |r| r.cells.len());
    let alignments = vec![TableAlignment::None; num_columns];

    let table = arena.alloc(
        NodeValue::Table(Box::new(comrak::nodes::NodeTable {
            alignments,
            num_columns,
            num_rows: table_content.rows.len(),
            num_nonempty_cells: 0,
        }))
        .into(),
    );

    for (i, table_row) in table_content.rows.iter().enumerate() {
        let is_header = i == 0
            && table_row
                .cells
                .first()
                .is_some_and(|c| c.cell_type == TableCellType::TableHeader);
        let row = arena.alloc(NodeValue::TableRow(is_header).into());
        for cell in &table_row.cells {
            let cell_node = arena.alloc(NodeValue::TableCell.into());
            append_inline_content(&cell.content, cell_node, arena);
            row.append(cell_node);
        }
        table.append(row);
    }

    parent.append(table);
}
