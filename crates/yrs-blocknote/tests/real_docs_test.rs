//! Batch round-trip tests using ALL real .md files from dev-notes/ and milestones/.
//!
//! Automatically discovers every .md file under these directories and runs:
//! - md → blocks → md (structural preservation)
//! - md → Y.Doc → md (structural preservation)
//! - double round-trip convergence

use std::path::{Path, PathBuf};

use yrs_blocknote::{
    blocks_to_markdown, doc_to_markdown, markdown_to_blocks, markdown_to_doc, BlockType,
};

// ── Helpers ──────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Recursively collect all .md files under a directory.
fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_md_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// Short display name for a path (relative to workspace root).
fn display_name(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// md → blocks → md: block count and types preserved.
fn assert_blocks_roundtrip(name: &str, md: &str) {
    let blocks = markdown_to_blocks(md);
    let output = blocks_to_markdown(&blocks).unwrap();

    assert!(!output.is_empty(), "[{name}] md→blocks→md produced empty output");

    let re_parsed = markdown_to_blocks(&output);
    assert_eq!(
        blocks.len(),
        re_parsed.len(),
        "[{name}] block count changed after blocks round-trip: {} → {}",
        blocks.len(),
        re_parsed.len(),
    );
    for (i, (orig, rt)) in blocks.iter().zip(re_parsed.iter()).enumerate() {
        assert_eq!(
            orig.block_type, rt.block_type,
            "[{name}] block[{i}] type changed: {:?} → {:?}",
            orig.block_type, rt.block_type,
        );
    }
}

/// md → Y.Doc → md: block count and types preserved.
fn assert_ydoc_roundtrip(name: &str, md: &str) {
    let doc = markdown_to_doc(md, "document-store");
    let output = doc_to_markdown(&doc, "document-store").unwrap();

    assert!(!output.is_empty(), "[{name}] md→ydoc→md produced empty output");

    let orig_blocks = markdown_to_blocks(md);
    let rt_blocks = markdown_to_blocks(&output);
    assert_eq!(
        orig_blocks.len(),
        rt_blocks.len(),
        "[{name}] block count changed after Y.Doc round-trip: {} → {}",
        orig_blocks.len(),
        rt_blocks.len(),
    );
    for (i, (orig, rt)) in orig_blocks.iter().zip(rt_blocks.iter()).enumerate() {
        assert_eq!(
            orig.block_type, rt.block_type,
            "[{name}] block[{i}] type changed: {:?} → {:?}",
            orig.block_type, rt.block_type,
        );
    }
}

/// Double round-trip: structure converges after 2nd pass.
fn assert_double_roundtrip_converges(name: &str, md: &str) {
    let blocks1 = markdown_to_blocks(md);
    let md1 = blocks_to_markdown(&blocks1).unwrap();
    let blocks2 = markdown_to_blocks(&md1);
    let md2 = blocks_to_markdown(&blocks2).unwrap();
    let blocks3 = markdown_to_blocks(&md2);

    assert_eq!(
        blocks2.len(),
        blocks3.len(),
        "[{name}] block count not stable after double round-trip: {} → {}",
        blocks2.len(),
        blocks3.len(),
    );
    for (i, (b2, b3)) in blocks2.iter().zip(blocks3.iter()).enumerate() {
        assert_eq!(
            b2.block_type, b3.block_type,
            "[{name}] block[{i}] type not stable: {:?} → {:?}",
            b2.block_type, b3.block_type,
        );
    }
}

// ── Batch tests ──────────────────────────────────────────

#[test]
fn batch_dev_notes_blocks_roundtrip() {
    let root = workspace_root();
    let files = collect_md_files(&root.join("dev-notes"));
    assert!(!files.is_empty(), "No .md files found in dev-notes/");

    let mut passed = 0;
    for path in &files {
        let name = display_name(path, &root);
        let md = std::fs::read_to_string(path).unwrap();
        if md.trim().is_empty() {
            continue;
        }
        assert_blocks_roundtrip(&name, &md);
        passed += 1;
    }
    eprintln!("dev-notes blocks round-trip: {passed}/{} files passed", files.len());
}

#[test]
fn batch_dev_notes_ydoc_roundtrip() {
    let root = workspace_root();
    let files = collect_md_files(&root.join("dev-notes"));
    assert!(!files.is_empty(), "No .md files found in dev-notes/");

    let mut passed = 0;
    for path in &files {
        let name = display_name(path, &root);
        let md = std::fs::read_to_string(path).unwrap();
        if md.trim().is_empty() {
            continue;
        }
        assert_ydoc_roundtrip(&name, &md);
        passed += 1;
    }
    eprintln!("dev-notes Y.Doc round-trip: {passed}/{} files passed", files.len());
}

#[test]
fn batch_milestones_blocks_roundtrip() {
    let root = workspace_root();
    let files = collect_md_files(&root.join("milestones"));
    assert!(!files.is_empty(), "No .md files found in milestones/");

    let mut passed = 0;
    for path in &files {
        let name = display_name(path, &root);
        let md = std::fs::read_to_string(path).unwrap();
        if md.trim().is_empty() {
            continue;
        }
        assert_blocks_roundtrip(&name, &md);
        passed += 1;
    }
    eprintln!("milestones blocks round-trip: {passed}/{} files passed", files.len());
}

#[test]
fn batch_milestones_ydoc_roundtrip() {
    let root = workspace_root();
    let files = collect_md_files(&root.join("milestones"));
    assert!(!files.is_empty(), "No .md files found in milestones/");

    let mut passed = 0;
    for path in &files {
        let name = display_name(path, &root);
        let md = std::fs::read_to_string(path).unwrap();
        if md.trim().is_empty() {
            continue;
        }
        assert_ydoc_roundtrip(&name, &md);
        passed += 1;
    }
    eprintln!("milestones Y.Doc round-trip: {passed}/{} files passed", files.len());
}

#[test]
fn batch_double_roundtrip_converges() {
    let root = workspace_root();
    let mut all_files = collect_md_files(&root.join("dev-notes"));
    all_files.extend(collect_md_files(&root.join("milestones")));
    assert!(!all_files.is_empty(), "No .md files found");

    let mut passed = 0;
    for path in &all_files {
        let name = display_name(path, &root);
        let md = std::fs::read_to_string(path).unwrap();
        if md.trim().is_empty() {
            continue;
        }
        assert_double_roundtrip_converges(&name, &md);
        passed += 1;
    }
    eprintln!("double round-trip convergence: {passed}/{} files passed", all_files.len());
}

// ── Content-specific checks ──────────────────────────────

/// Verify tables survive round-trip across ALL docs that contain them.
#[test]
fn batch_tables_preserved() {
    let root = workspace_root();
    let mut all_files = collect_md_files(&root.join("dev-notes"));
    all_files.extend(collect_md_files(&root.join("milestones")));

    let mut docs_with_tables = 0;
    for path in &all_files {
        let md = std::fs::read_to_string(path).unwrap();
        let blocks = markdown_to_blocks(&md);
        let table_count = blocks.iter().filter(|b| b.block_type == BlockType::Table).count();
        if table_count == 0 {
            continue;
        }

        docs_with_tables += 1;
        let name = display_name(path, &root);

        // blocks round-trip
        let output = blocks_to_markdown(&blocks).unwrap();
        let rt_blocks = markdown_to_blocks(&output);
        let rt_table_count = rt_blocks.iter().filter(|b| b.block_type == BlockType::Table).count();
        assert_eq!(
            table_count, rt_table_count,
            "[{name}] table count changed after blocks round-trip: {table_count} → {rt_table_count}"
        );

        // Y.Doc round-trip
        let doc = markdown_to_doc(&md, "document-store");
        let ydoc_blocks = yrs_blocknote::doc_to_blocks(&doc, "document-store").unwrap();
        let ydoc_table_count = ydoc_blocks.iter().filter(|b| b.block_type == BlockType::Table).count();
        assert_eq!(
            table_count, ydoc_table_count,
            "[{name}] table count changed after Y.Doc round-trip: {table_count} → {ydoc_table_count}"
        );
    }
    assert!(docs_with_tables > 0, "Expected at least one doc with tables");
    eprintln!("tables preserved in {docs_with_tables} documents");
}

/// Verify code blocks (with language tags) survive round-trip.
#[test]
fn batch_code_blocks_preserved() {
    let root = workspace_root();
    let mut all_files = collect_md_files(&root.join("dev-notes"));
    all_files.extend(collect_md_files(&root.join("milestones")));

    let mut docs_with_code = 0;
    for path in &all_files {
        let md = std::fs::read_to_string(path).unwrap();
        let blocks = markdown_to_blocks(&md);
        let code_count = blocks.iter().filter(|b| b.block_type == BlockType::CodeBlock).count();
        if code_count == 0 {
            continue;
        }

        docs_with_code += 1;
        let name = display_name(path, &root);

        let doc = markdown_to_doc(&md, "document-store");
        let rt_blocks = yrs_blocknote::doc_to_blocks(&doc, "document-store").unwrap();
        let rt_code_count = rt_blocks.iter().filter(|b| b.block_type == BlockType::CodeBlock).count();
        assert_eq!(
            code_count, rt_code_count,
            "[{name}] code block count changed after Y.Doc round-trip: {code_count} → {rt_code_count}"
        );
    }
    assert!(docs_with_code > 0, "Expected at least one doc with code blocks");
    eprintln!("code blocks preserved in {docs_with_code} documents");
}
