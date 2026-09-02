//! Bridge Hermes → Redox cookbook — promoção HITL + build pkgutils (Onda 7f / Fase 2).

use std::path::{Path, PathBuf};
use std::process::Command;

use agent_core::allows_pkg_install;
use skill_registry::{DynamicSkill, SkillStage};

use crate::hitl::{gate_response, hitl_enabled};

#[derive(Clone, Debug)]
pub struct PromoteResult {
    pub recipe_path: PathBuf,
    pub pkgutils_cmd: String,
    pub package_path: Option<PathBuf>,
}

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

/// Comando pkgutils/cookbook sugerido para build da recipe staged.
pub fn pkgutils_build_command(recipe_path: &Path) -> String {
    let recipe_dir = recipe_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ".".into());
    let name = recipe_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("skill");
    let redox_root = std::env::var("REDOX_ROOT").unwrap_or_else(|_| "/path/to/redox".into());
    format!(
        "cd {redox_root} && cookbook build {recipe_dir}\n# ou: make r.recipe-aios-skills-{name}"
    )
}

fn cookbook_build_enabled() -> bool {
    std::env::var("REDOX_COOKBOOK_BUILD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Tenta build via `cookbook` quando `REDOX_COOKBOOK_BUILD=1` (Linux/WSL/Redox).
pub fn try_pkgutils_build(recipe_path: &Path) -> Result<Option<PathBuf>, String> {
    if !cookbook_build_enabled() {
        return Ok(None);
    }
    let recipe_dir = recipe_path
        .parent()
        .ok_or_else(|| "recipe sem diretório".to_string())?;
    let output = Command::new("cookbook")
        .arg("build")
        .arg(recipe_dir)
        .output()
        .map_err(|e| format!("cookbook build: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cookbook build falhou: {stderr}"));
    }
    Ok(Some(recipe_dir.join("target").join("bin")))
}

/// Promove skill WASM persistente para recipe Redox cookbook (requer HITL + grant).
pub fn promote_skill_to_recipe(
    skill: &DynamicSkill,
    hitl_approved: bool,
) -> Result<PromoteResult, String> {
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

    if !allows_pkg_install(hitl_approved) {
        return Err(
            "promoção requer grant pkg_install ou hitl_approve (REDOX_AIOS_CAPS)".into(),
        );
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

    let recipe_path = skill_dir.join("recipe.toml");
    let pkgutils_cmd = pkgutils_build_command(&recipe_path);
    let package_path = try_pkgutils_build(&recipe_path).ok().flatten();

    Ok(PromoteResult {
        recipe_path,
        pkgutils_cmd,
        package_path,
    })
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
        std::env::set_var("REDOX_AIOS_CAPS", "hitl_approve,pkg_install,factory_exec");
        std::env::set_var("REDOX_RECIPES_STAGING", &staging);

        let mut skill = DynamicSkill::with_wasm(
            "test_skill_stage",
            "test",
            "return 42",
            echo_len_module(),
            "run",
        );
        skill.stage = SkillStage::WasmPersistent;

        let result = promote_skill_to_recipe(&skill, true).expect("promote");
        assert!(result.recipe_path.is_file());
        assert!(result
            .recipe_path
            .parent()
            .unwrap()
            .join("test_skill_stage.wasm")
            .is_file());
        assert!(result.pkgutils_cmd.contains("cookbook build"));

        match prev_hitl {
            Some(v) => std::env::set_var("REDOX_HERMES_HITL", v),
            None => std::env::remove_var("REDOX_HERMES_HITL"),
        }
        std::env::remove_var("REDOX_AIOS_CAPS");
        match prev_staging {
            Some(v) => std::env::set_var("REDOX_RECIPES_STAGING", v),
            None => std::env::remove_var("REDOX_RECIPES_STAGING"),
        }
    }
}
