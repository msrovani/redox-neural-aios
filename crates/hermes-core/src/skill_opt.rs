//! SkillOpt — promoção SkillMd → WASM → file (ADR-010 Onda 7h).

use skill_registry::{persist_wasm, DynamicSkill, SkillStage};

/// Compila wasm em memória quando skill SKILL.md madura.
pub fn maybe_compile_wasm(
    skill: &mut DynamicSkill,
    wasm: Vec<u8>,
    export_fn: &str,
) -> Option<String> {
    if !skill.eligible_for_wasm_compile() {
        return None;
    }
    skill.attach_wasm(wasm, export_fn);
    Some(format!(
        "compiled {} → wasm memory (runs={}, rate={:.0}%)",
        skill.name,
        skill.runs,
        skill.success_rate * 100.0
    ))
}

/// Persiste `.wasm` quando bytecode já está em memória.
pub fn maybe_persist_wasm(skill: &mut DynamicSkill) -> Option<String> {
    if !skill.eligible_for_file_promotion() {
        return None;
    }
    let wasm = skill.wasm.clone()?;
    let path = persist_wasm(&skill.name, &wasm).ok()?;
    skill.promote_wasm_file(wasm, skill.export_fn.clone());
    Some(format!(
        "promoted {} → {} (stage={:?})",
        skill.name,
        path.display(),
        skill.stage
    ))
}

pub fn stage_label(stage: SkillStage) -> &'static str {
    match stage {
        SkillStage::SkillMd => "skill_md",
        SkillStage::WasmMemory => "wasm_memory",
        SkillStage::WasmPersistent => "wasm_file",
    }
}
