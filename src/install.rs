use anyhow::Result;
use std::path::Path;

struct SkillTemplate {
    filename: &'static str,
    content: &'static str,
}

const SKILLS: &[SkillTemplate] = &[
    SkillTemplate {
        filename: "ra-hover.md",
        content: include_str!("skills/ra-hover.md"),
    },
    SkillTemplate {
        filename: "ra-definition.md",
        content: include_str!("skills/ra-definition.md"),
    },
    SkillTemplate {
        filename: "ra-references.md",
        content: include_str!("skills/ra-references.md"),
    },
    SkillTemplate {
        filename: "ra-search.md",
        content: include_str!("skills/ra-search.md"),
    },
    SkillTemplate {
        filename: "ra-diagnostics.md",
        content: include_str!("skills/ra-diagnostics.md"),
    },
    SkillTemplate {
        filename: "ra-workspace-diagnostics.md",
        content: include_str!("skills/ra-workspace-diagnostics.md"),
    },
    SkillTemplate {
        filename: "ra-callers.md",
        content: include_str!("skills/ra-callers.md"),
    },
    SkillTemplate {
        filename: "ra-callees.md",
        content: include_str!("skills/ra-callees.md"),
    },
    SkillTemplate {
        filename: "ra-implementations.md",
        content: include_str!("skills/ra-implementations.md"),
    },
    SkillTemplate {
        filename: "ra-setup.md",
        content: include_str!("skills/ra-setup.md"),
    },
    SkillTemplate {
        filename: "ra-impact.md",
        content: include_str!("skills/ra-impact.md"),
    },
];

const CLAUDE_MD_SECTION_MARKER: &str = "<!-- rust-analyzer-server -->";

const CLAUDE_MD_SNIPPET: &str = r#"<!-- rust-analyzer-server -->
## rust-analyzer Server (Code Intelligence)

A rust-analyzer HTTP server provides LSP-powered code intelligence. **Prefer these tools over Grep/Glob for code structure queries.**

### Server Info

- **Port**: `15423` (default, override with `RUST_ANALYZER_PORT` env var)
- **Health**: `curl -s http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/health`
- **Status**: `curl -s http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/status`

### Starting the Server

```bash
# Start (keeps rust-analyzer warm across requests)
nohup rust-analyzer-server --workspace /path/to/this/project > /tmp/rust-analyzer-server.log 2>&1 &

# Custom port
nohup rust-analyzer-server --workspace /path/to/this/project --port 4000 > /tmp/rust-analyzer-server.log 2>&1 &
```

### Available Skills (slash commands)

| Command | Purpose | Example |
|---------|---------|---------|
| `/ra-setup [path]` | Health check + set workspace | `/ra-setup /path/to/project` |
| `/ra-hover` | Type info + docs | `/ra-hover src/main.rs 5 10` |
| `/ra-definition` | Go to definition | `/ra-definition src/main.rs 5 10` |
| `/ra-references` | Find all usages | `/ra-references src/main.rs 5 10` |
| `/ra-search` | Workspace symbol search | `/ra-search MyStruct` |
| `/ra-diagnostics` | File errors/warnings | `/ra-diagnostics src/main.rs` |
| `/ra-workspace-diagnostics` | All project diagnostics | `/ra-workspace-diagnostics` |
| `/ra-callers` | Who calls this function? | `/ra-callers src/main.rs 10 4` |
| `/ra-callees` | What does this call? | `/ra-callees src/main.rs 10 4` |
| `/ra-implementations` | Trait implementations | `/ra-implementations src/main.rs 5 10` |
| `/ra-impact` | Change impact analysis | `/ra-impact src/main.rs 10 4` |

### Recommended Workflow

```
1. /ra-setup              -> verify server is running
2. /ra-search MyFunction  -> find symbol location
3. Read file              -> read the code
4. /ra-hover ...          -> check types of external symbols
5. /ra-definition ...     -> jump to definitions
6. /ra-references ...     -> find all usages (impact analysis)
7. /ra-callers ...        -> trace call hierarchy
8. /ra-diagnostics ...    -> check for errors
```

### When to Use What

- **Code structure** (functions, types, call graphs): Use `/ra-*` skills
- **Text search** (string literals, comments, config): Use Grep/Glob
<!-- /rust-analyzer-server -->"#;

pub fn install_skills(target: &Path) -> Result<()> {
    let commands_dir = target.join(".claude").join("commands");
    std::fs::create_dir_all(&commands_dir)?;

    let mut installed = Vec::new();

    for skill in SKILLS {
        let dest = commands_dir.join(skill.filename);
        std::fs::write(&dest, skill.content)?;
        installed.push(skill.filename);
    }

    eprintln!("Installed {} skills into {}", installed.len(), commands_dir.display());
    for name in &installed {
        eprintln!("  /{}", name.strip_suffix(".md").unwrap_or(name));
    }

    // Append rust-analyzer guide to CLAUDE.md
    install_claude_md_section(target)?;

    Ok(())
}

fn install_claude_md_section(target: &Path) -> Result<()> {
    let claude_md = target.join("CLAUDE.md");

    if claude_md.exists() {
        let content = std::fs::read_to_string(&claude_md)?;

        // Already has our section — replace it
        if content.contains(CLAUDE_MD_SECTION_MARKER) {
            let start = content.find(CLAUDE_MD_SECTION_MARKER).unwrap();
            let end_marker = "<!-- /rust-analyzer-server -->";
            let end = content
                .find(end_marker)
                .map(|i| i + end_marker.len())
                .unwrap_or(content.len());

            let mut new_content = String::new();
            new_content.push_str(&content[..start]);
            new_content.push_str(CLAUDE_MD_SNIPPET);
            new_content.push_str(&content[end..]);
            std::fs::write(&claude_md, new_content)?;
            eprintln!("Updated rust-analyzer section in {}", claude_md.display());
        } else {
            // Append to existing CLAUDE.md
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(CLAUDE_MD_SNIPPET);
            content.push('\n');
            std::fs::write(&claude_md, content)?;
            eprintln!("Appended rust-analyzer section to {}", claude_md.display());
        }
    } else {
        // Create new CLAUDE.md
        let content = format!("# CLAUDE.md\n\n{}\n", CLAUDE_MD_SNIPPET);
        std::fs::write(&claude_md, content)?;
        eprintln!("Created {} with rust-analyzer guide", claude_md.display());
    }

    Ok(())
}
