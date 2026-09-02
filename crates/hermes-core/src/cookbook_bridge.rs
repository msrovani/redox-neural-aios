//! Bridge Hermes → Redox cookbook — promoção HITL de skills maduras (Onda 7f).

use std::path::{Path, PathBuf};

use skill_registry::{DynamicSkill, SkillStage};

use crate::hitl::{gate_response, hitl_enabled};

pub fn recipes_staging_dir() -> PathBuf {
    std::env::var("REDOX_RECIPES_STAGING")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../recipes/aios/skills")
            } else {
                PathBuf::from("/usr/share/aios/recipes/skills-staging")
            }
        })
}

fn sanitize_name(name: &str) -> Result<String, String> {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() || clean.chars().all(|c| c == '_') {
        return Err("nome de skill inválido para recipe".into());
    }
    Ok(clean)
}

fn write_recipe_toml(skill_dir: &Path, skill_name: &str) -> Result<(), String> {
    let recipe = format!(
        r#"# Skill WASM auto-gerada — promoção HITL (Onda 7f ADR-010)
# Instala em /usr/lib/aios/skills/{skill_name}.wasm

[source]
path = "."

[build]
template = "custom"
script = """
mkdir -pv "$BUILD/usr/lib/aios/skills"
cp "$SRC/{skill_name}.wasm" "$BUILD/usr/lib/aios/skills/"
"""
"#
    );
    std::fs::write(skill_dir.join("recipe.toml"), recipe)
        .map_err(|e| format!("write recipe.toml: {e}"))
}

fn write_manifest(skill_dir: &Path, skill: &DynamicSkill) -> Result<(), String> {
    let manifest = format!(
        "name={}\ndesc={}\nauthor=Hermes Runtime App Factory\nversion=0.1.0\n",
        skill.name, skill.description
    );
    std::fs::write(skill_dir.join("manifest"), manifest)
        .map_err(|e| format!("write manifest: {e}"))
}

/// Promove skill WASM persistente para recipe Redox cookbook (requer HITL).
pub fn promote_skill_to_recipe(skill: &DynamicSkill, hitl_approved: bool) -> Result<PathBuf, String> {
    if skill.stage != SkillStage::WasmPersistent {
        return Err("skill precisa estar em stage WasmPersistent".into());
    }
    let wasm = skill
        .wasm
        .as_ref()
        .ok_or_else(|| "skill sem bytecode WASM".to_string())?;

    if hitl_enabled() && !hitl_approved {
        return Err(i18n_core::t("factory.promote.hitl"));
    }

    if let Some(blocked) = gate_response(&format!("promote skill {} to os package", skill.name)) {
        return Err(format!("{}: {blocked}", i18n_core::t("factory.promote.blocked")));
    }

    let name = sanitize_name(&skill.name)?;
    let skill_dir = recipes_staging_dir().join(&name);
    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("mkdir recipe staging: {e}"))?;

    let wasm_path = skill_dir.join(format!("{name}.wasm"));
    std::fs::write(&wasm_path, wasm).map_err(|e| format!("write wasm: {e}"))?;
    write_recipe_toml(&skill_dir, &name)?;
    write_manifest(&skill_dir, skill)?;

    Ok(skill_dir.join("recipe.toml"))
}

pub fn list_staged_recipes() -> Vec<PathBuf> {
    let dir = recipes_staging_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("recipe.toml").is_file())
        .map(|e| e.path().join("recipe.toml"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use skill_registry::DynamicSkill;
    use wasm_skill_runtime::echo_len_module;

    #[test]
    fn stages_recipe_when_approved() {
        let prev_hitl = std::env::var("REDOX_HERMES_HITL").ok();
        let staging = std::env::temp_dir()
            .join("redox-aios-recipe-test")
            .to_string_lossy()
            .into_owned();
        let prev_staging = std::env::var("REDOX_RECIPES_STAGING").ok();
        std::env::set_var("REDOX_HERMES_HITL", "0");
        std::env::set_var("REDOX_RECIPES_STAGING", &staging);

        let mut skill = DynamicSkill::with_wasm(
            "test_skill_stage",
            "test",
            "return 42",
            echo_len_module(),
            "run",
        );
        skill.stage = SkillStage::WasmPersistent;

        let path = promote_skill_to_recipe(&skill, true).expect("promote");
        assert!(path.is_file());
        assert!(path.parent().unwrap().join("test_skill_stage.wasm").is_file());

        match prev_hitl {
            Some(v) => std::env::set_var("REDOX_HERMES_HITL", v),
            None => std::env::remove_var("REDOX_HERMES_HITL"),
        }
        match prev_staging {
            Some(v) => std::env::set_var("REDOX_RECIPES_STAGING", v),
            None => std::env::remove_var("REDOX_RECIPES_STAGING"),
        }
    }
}
