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
        eprintln!("  /{}",  name.strip_suffix(".md").unwrap_or(name));
    }

    Ok(())
}
