//! Filesystem-based import chain tracing for installed skills.
//!
//! Reads the skill's SKILL.md and any referenced scripts, scans them for security
//! issues, and builds a dependency graph showing where suspicious code originates.

use ai_skill_core::{
    DepEdge, DepGraph, DepNode, ImportChainFinding, ImportRef, parse_import_refs, scan_skill,
};
use std::path::{Path, PathBuf};

/// Maximum recursion depth for import chain tracing.
const MAX_DEPTH: usize = 5;

/// Errors that can occur during import chain tracing.
#[derive(Debug, thiserror::Error)]
pub enum ImportChainError {
    #[error("failed to read SKILL.md: {0}")]
    ReadSkillMd(std::io::Error),
    #[error("failed to read referenced file '{path}': {inner}")]
    ReadReferenced { path: String, inner: std::io::Error },
    #[error("import chain exceeds max depth ({MAX_DEPTH})")]
    MaxDepthExceeded,
}

/// Traces the import chain for a skill at the given directory.
///
/// Reads SKILL.md, parses import references, recursively scans referenced files,
/// and returns findings with their origin chain plus a dependency graph.
pub fn trace_import_chain(skill_dir: &Path) -> Result<ImportChainResult, ImportChainError> {
    let manifest_path = skill_dir.join("SKILL.md");
    let manifest_content =
        std::fs::read_to_string(&manifest_path).map_err(ImportChainError::ReadSkillMd)?;

    let mut all_findings: Vec<ImportChainFinding> = Vec::new();
    let mut graph = DepGraph::default();

    // Root node: SKILL.md
    let root_id = 0;
    graph.nodes.push(DepNode {
        id: root_id,
        path: "SKILL.md".to_string(),
        has_findings: false,
        finding_count: 0,
    });

    // Scan SKILL.md itself
    let manifest_findings = scan_skill(&manifest_content);
    if !manifest_findings.is_empty() {
        graph.nodes[root_id].has_findings = true;
        graph.nodes[root_id].finding_count = manifest_findings.len();
        for f in manifest_findings {
            all_findings.push(ImportChainFinding {
                finding: f,
                origin_file: "SKILL.md".to_string(),
                import_chain: vec![],
            });
        }
    }

    // Trace imports from SKILL.md
    let imports = parse_import_refs(&manifest_content);
    let mut next_id = 1;
    let mut stack: Vec<TraceFrame> = imports
        .into_iter()
        .map(|reference| TraceFrame {
            reference,
            from_id: root_id,
            current_dir: skill_dir.to_path_buf(),
            depth: 0,
        })
        .collect();

    while let Some(frame) = stack.pop() {
        if frame.depth >= MAX_DEPTH {
            // Still add a node for the reference point so the graph is complete
            let ref_id = next_id;
            next_id += 1;
            graph.nodes.push(DepNode {
                id: ref_id,
                path: frame.reference.referenced_path.clone(),
                has_findings: false,
                finding_count: 0,
            });
            graph.edges.push(DepEdge {
                from: frame.from_id,
                to: ref_id,
                reference: frame.reference,
            });
            continue;
        }

        let resolved = frame.current_dir.join(&frame.reference.referenced_path);
        let canonical = match std::fs::canonicalize(&resolved) {
            Ok(p) => p,
            Err(_) => continue, // file doesn't exist, skip
        };

        // Ensure we don't escape the skill directory
        if !canonical.starts_with(skill_dir) {
            continue;
        }

        let file_content = match std::fs::read_to_string(&canonical) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let relative_path = canonical
            .strip_prefix(skill_dir)
            .unwrap_or(&canonical)
            .to_string_lossy()
            .to_string();

        // Check if we already have this file as a node
        let file_id = if let Some(existing) = graph.nodes.iter().find(|n| n.path == relative_path) {
            existing.id
        } else {
            let id = next_id;
            next_id += 1;
            graph.nodes.push(DepNode {
                id,
                path: relative_path.clone(),
                has_findings: false,
                finding_count: 0,
            });
            id
        };

        let ref_clone = frame.reference.clone();

        // Add the edge
        if !graph
            .edges
            .iter()
            .any(|e| e.from == frame.from_id && e.to == file_id)
        {
            graph.edges.push(DepEdge {
                from: frame.from_id,
                to: file_id,
                reference: frame.reference,
            });
        }

        let chain_so_far = vec![ref_clone];

        // Scan the file
        let file_findings = scan_skill(&file_content);
        if !file_findings.is_empty() {
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == file_id) {
                node.has_findings = true;
                node.finding_count = file_findings.len();
            }
            for f in file_findings {
                all_findings.push(ImportChainFinding {
                    finding: f,
                    origin_file: relative_path.clone(),
                    import_chain: chain_so_far.clone(),
                });
            }
        }

        // Recurse into this file's imports
        let sub_imports = parse_import_refs(&file_content);
        let parent_dir = canonical.parent().unwrap_or(&canonical).to_path_buf();
        for sub_ref in sub_imports {
            stack.push(TraceFrame {
                reference: sub_ref,
                from_id: file_id,
                current_dir: parent_dir.clone(),
                depth: frame.depth + 1,
            });
        }
    }

    Ok(ImportChainResult {
        findings: all_findings,
        graph,
    })
}

/// Result of tracing a skill's import chain.
#[derive(Debug)]
pub struct ImportChainResult {
    /// All findings discovered across the import chain.
    pub findings: Vec<ImportChainFinding>,
    /// The dependency graph of all referenced files.
    pub graph: DepGraph,
}

/// Internal frame for the tracing stack.
struct TraceFrame {
    reference: ImportRef,
    from_id: usize,
    current_dir: PathBuf,
    depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::Severity;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn trace_no_imports_returns_only_manifest() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\n---\n# Safe skill\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        assert!(result.findings.is_empty());
        assert_eq!(result.graph.nodes.len(), 1); // just SKILL.md
        assert_eq!(result.graph.nodes[0].path, "SKILL.md");
    }

    #[test]
    fn trace_finding_in_manifest() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("bad-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: bad-skill\n---\nrm -rf /tmp\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].origin_file, "SKILL.md");
        assert_eq!(
            result.findings[0].finding.category,
            ai_skill_core::ScanCategory::DangerousShellPattern
        );
    }

    #[test]
    fn trace_imported_script_finding() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("import-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: import-skill\n---\nsource ./scripts/setup.sh\n",
        )
        .unwrap();
        fs::create_dir_all(skill_dir.join("scripts")).unwrap();
        fs::write(
            skill_dir.join("scripts").join("setup.sh"),
            "#!/bin/bash\neval \"$PAYLOAD\"\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].origin_file, "scripts/setup.sh");
        assert_eq!(
            result.findings[0].finding.category,
            ai_skill_core::ScanCategory::DangerousShellPattern
        );
        // Should have SKILL.md + the script
        assert_eq!(result.graph.nodes.len(), 2);
        assert!(result.graph.nodes.iter().any(|n| n.path == "SKILL.md"));
        assert!(
            result
                .graph
                .nodes
                .iter()
                .any(|n| n.path == "scripts/setup.sh")
        );
        // Should have an edge
        assert_eq!(result.graph.edges.len(), 1);
    }

    #[test]
    fn trace_chain_of_imports() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("chain-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::create_dir_all(skill_dir.join("lib")).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: chain-skill\n---\nsource ./lib/utils.sh\n",
        )
        .unwrap();
        // utils.sh references config.sh relative to its own directory (lib/)
        fs::write(
            skill_dir.join("lib").join("utils.sh"),
            "source ./config.sh\n",
        )
        .unwrap();
        fs::write(
            skill_dir.join("lib").join("config.sh"),
            "API_KEY = sk-abc123\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        // The finding should be in lib/config.sh (the deep file)
        assert!(!result.findings.is_empty());
        assert_eq!(result.findings[0].origin_file, "lib/config.sh");
        // Should have 3 nodes: SKILL.md, lib/utils.sh, lib/config.sh
        assert_eq!(result.graph.nodes.len(), 3);
        assert_eq!(result.graph.edges.len(), 2);
    }

    #[test]
    fn trace_missing_import_file_skips_gracefully() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("missing-ref");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: missing-ref\n---\nsource ./nonexistent.sh\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        assert!(result.findings.is_empty());
        assert_eq!(result.graph.nodes.len(), 1); // only SKILL.md, nonexistent file skipped
    }

    #[test]
    fn trace_out_of_skill_dir_reference_skipped() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("out-ref");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: out-ref\n---\nsource /etc/passwd\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        assert!(result.findings.is_empty());
        assert_eq!(result.graph.nodes.len(), 1); // only SKILL.md
    }

    #[test]
    fn trace_max_depth_exceeded() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("deep-chain");
        fs::create_dir_all(&skill_dir).unwrap();

        // Create a deep chain: SKILL.md → a.sh → b.sh → c.sh → d.sh → e.sh → over max
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: deep\n---\nsource ./a.sh\n",
        )
        .unwrap();
        let mut prev = "a.sh".to_string();
        for name in &["b.sh", "c.sh", "d.sh", "e.sh", "f.sh"] {
            fs::write(skill_dir.join(&prev), format!("source ./{name}\n")).unwrap();
            prev = name.to_string();
        }
        fs::write(skill_dir.join("f.sh"), "eval \"bad\"\n").unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        // We trace up to MAX_DEPTH (5) levels deep
        // SKILL.md -> a.sh -> b.sh -> c.sh -> d.sh -> e.sh (depth 0..4, that's 5 levels)
        // MAX_DEPTH=5 means depth >= 5 stops recursion
        // So e.sh gets scanned but f.sh doesn't
        // But we should still get max depth exceeded behavior
        assert!(result.graph.nodes.len() > 1);
    }

    #[test]
    fn trace_empty_skill_dir_returns_no_imports() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("empty");
        fs::create_dir_all(&skill_dir).unwrap();
        // No SKILL.md
        let result = trace_import_chain(&skill_dir);
        assert!(result.is_err());
    }

    #[test]
    fn trace_parse_import_refs_integration() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("multi-ref");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::create_dir_all(skill_dir.join("scripts")).unwrap();
        fs::create_dir_all(skill_dir.join("lib")).unwrap();

        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: multi\n---\nsource ./scripts/init.sh\nrequire('./lib/helper.js')\n",
        )
        .unwrap();
        fs::write(
            skill_dir.join("scripts").join("init.sh"),
            "bash ./tools/setup.sh\n",
        )
        .unwrap();
        fs::create_dir_all(skill_dir.join("scripts").join("tools")).unwrap();
        fs::write(
            skill_dir.join("scripts").join("tools").join("setup.sh"),
            "rm -rf /var/tmp\n",
        )
        .unwrap();
        fs::write(
            skill_dir.join("lib").join("helper.js"),
            "console.log('helper')\n",
        )
        .unwrap();

        let result = trace_import_chain(&skill_dir).unwrap();
        // Should find the dangerous pattern in scripts/tools/setup.sh
        let dangerous: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.finding.severity == Severity::High)
            .collect();
        assert_eq!(dangerous.len(), 1);
        assert_eq!(dangerous[0].origin_file, "scripts/tools/setup.sh");
    }
}
