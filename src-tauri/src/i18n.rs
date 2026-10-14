//! Mensajes de error/validación bilingües que ve el usuario. El texto de los
//! prompts para la IA (assistant.rs: schemas y briefings) queda aparte — es
//! instrucción para el modelo, no UI, así que no vive acá.
use crate::models::Lang;

pub fn read_file_failed(lang: Lang, name: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't read {name}: {err}"),
        Lang::Es => format!("No pude leer {name}: {err}"),
    }
}

pub fn agent_already_exists(lang: Lang, slug: &str) -> String {
    match lang {
        Lang::En => format!(
            "An agent \"{slug}\" already exists in this scope. Open it from the list to edit it, or pick another name."
        ),
        Lang::Es => format!(
            "Ya existe un agente \"{slug}\" en este alcance. Ábrelo desde la lista si quieres editarlo, o elige otro nombre."
        ),
    }
}

pub fn skill_already_exists(lang: Lang, slug: &str) -> String {
    match lang {
        Lang::En => format!(
            "A skill \"{slug}\" already exists in this scope. Open it from the list to edit it, or pick another name."
        ),
        Lang::Es => format!(
            "Ya existe un skill \"{slug}\" en este alcance. Ábrelo desde la lista si quieres editarlo, o elige otro nombre."
        ),
    }
}

pub fn rule_already_exists(lang: Lang, slug: &str) -> String {
    match lang {
        Lang::En => format!(
            "A rule \"{slug}\" already exists in this scope. Open it from the list to edit it, or pick another name."
        ),
        Lang::Es => format!(
            "Ya existe una regla \"{slug}\" en este alcance. Ábrela desde la lista si quieres editarla, o elige otro nombre."
        ),
    }
}

pub fn no_skill_md_found(lang: Lang, id: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't find SKILL.md in {id}/"),
        Lang::Es => format!("No encontré SKILL.md en {id}/"),
    }
}

pub fn path_outside_rules_dir(lang: Lang) -> String {
    match lang {
        Lang::En => "Path is outside the rules folder.".into(),
        Lang::Es => "Ruta fuera de la carpeta de reglas.".into(),
    }
}

pub fn mcp_server_needs_name(lang: Lang) -> String {
    match lang {
        Lang::En => "Give the MCP server a name.".into(),
        Lang::Es => "Ponle un nombre al servidor MCP.".into(),
    }
}

pub fn mcp_server_needs_url(lang: Lang) -> String {
    match lang {
        Lang::En => "This server type needs a URL.".into(),
        Lang::Es => "Este tipo de servidor necesita una URL.".into(),
    }
}

pub fn mcp_server_needs_command(lang: Lang) -> String {
    match lang {
        Lang::En => "A stdio server needs a command.".into(),
        Lang::Es => "Un servidor stdio necesita un comando.".into(),
    }
}

pub fn no_mcp_json_in_repo(lang: Lang) -> String {
    match lang {
        Lang::En => "I couldn't find any .mcp.json in that repo.".into(),
        Lang::Es => "No pude encontrar ningún .mcp.json en ese repo.".into(),
    }
}

pub fn mcp_json_disappeared(lang: Lang, rel_path: &str) -> String {
    match lang {
        Lang::En => format!("{rel_path} no longer has that server — did the repo change?"),
        Lang::Es => format!("{rel_path} ya no tiene ese servidor — ¿cambió el repo?"),
    }
}

pub fn output_style_already_exists(lang: Lang, slug: &str) -> String {
    match lang {
        Lang::En => format!(
            "An output style \"{slug}\" already exists in this scope. Open it from the list to edit it, or pick another name."
        ),
        Lang::Es => format!(
            "Ya existe un output style \"{slug}\" en este alcance. Ábrelo desde la lista si quieres editarlo, o elige otro nombre."
        ),
    }
}

pub fn path_outside_output_styles_dir(lang: Lang) -> String {
    match lang {
        Lang::En => "Path is outside the output-styles folder.".into(),
        Lang::Es => "Ruta fuera de la carpeta de output-styles.".into(),
    }
}

pub fn invalid_path(lang: Lang) -> String {
    match lang {
        Lang::En => "Invalid path.".into(),
        Lang::Es => "Ruta inválida.".into(),
    }
}

pub fn path_outside_allowed_dir(lang: Lang) -> String {
    match lang {
        Lang::En => "Path is outside the allowed folder.".into(),
        Lang::Es => "Ruta fuera de la carpeta permitida.".into(),
    }
}

pub fn not_a_directory(lang: Lang, path: &str) -> String {
    match lang {
        Lang::En => format!("\"{path}\" is not a folder."),
        Lang::Es => format!("\"{path}\" no es una carpeta."),
    }
}

pub fn create_dir_failed(lang: Lang, path: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't create folder {path}: {err}"),
        Lang::Es => format!("No pude crear la carpeta {path}: {err}"),
    }
}

pub fn resolve_path_failed(lang: Lang, path: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't resolve {path}: {err}"),
        Lang::Es => format!("No pude resolver {path}: {err}"),
    }
}

pub fn git_not_found(lang: Lang) -> String {
    match lang {
        Lang::En => "I couldn't find \"git\" on your PATH — it's needed to clone the repo.".into(),
        Lang::Es => "No encontré \"git\" en tu PATH — hace falta para clonar el repo.".into(),
    }
}

pub fn git_exec_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't run git: {err}"),
        Lang::Es => format!("No pude ejecutar git: {err}"),
    }
}

pub fn git_clone_failed(lang: Lang, stderr: &str) -> String {
    match lang {
        Lang::En => format!("git clone failed: {stderr}"),
        Lang::Es => format!("git clone falló: {stderr}"),
    }
}

pub fn no_skill_md_in_repo(lang: Lang) -> String {
    match lang {
        Lang::En => "I couldn't find any SKILL.md in that repo.".into(),
        Lang::Es => "No encontré ningún SKILL.md en ese repo.".into(),
    }
}

pub fn skill_md_disappeared(lang: Lang, rel_path: &str) -> String {
    match lang {
        Lang::En => format!("{rel_path} no longer has a SKILL.md — did the repo change?"),
        Lang::Es => format!("{rel_path} ya no tiene un SKILL.md — ¿cambió el repo?"),
    }
}

pub fn skill_already_exists_on_import(lang: Lang, folder_name: &str) -> String {
    match lang {
        Lang::En => format!(
            "A skill \"{folder_name}\" already exists in this scope — remove it first, or install into the other scope (Project/Global)."
        ),
        Lang::Es => format!(
            "Ya existe un skill \"{folder_name}\" en este alcance — elimínalo primero o instálalo en el otro alcance (Proyecto/Global)."
        ),
    }
}

pub fn marketplace_request_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't reach the skills marketplace (skills.sh): {err}"),
        Lang::Es => format!("No pude conectar con el marketplace de skills (skills.sh): {err}"),
    }
}

pub fn github_request_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't reach GitHub: {err}"),
        Lang::Es => format!("No pude conectar con GitHub: {err}"),
    }
}

pub fn skill_repo_gone(lang: Lang, source: &str) -> String {
    match lang {
        Lang::En => format!(
            "GitHub has no \"{source}\" — it may have been deleted, renamed or made private since skills.sh indexed it."
        ),
        Lang::Es => format!(
            "GitHub no tiene \"{source}\" — puede que lo hayan borrado, renombrado o puesto privado desde que skills.sh lo indexó."
        ),
    }
}

pub fn skill_not_found_in_repo(lang: Lang, skill_id: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't find skill \"{skill_id}\" in that repo — did it move?"),
        Lang::Es => format!("No encontré el skill \"{skill_id}\" en ese repo — ¿se movió?"),
    }
}

pub fn hook_group_not_array(lang: Lang, event: &str) -> String {
    match lang {
        Lang::En => format!("\"{event}\" should be a list of groups."),
        Lang::Es => format!("\"{event}\" debería ser una lista de grupos."),
    }
}

pub fn hook_group_missing_hooks(lang: Lang, event: &str) -> String {
    match lang {
        Lang::En => format!("A group under \"{event}\" has no \"hooks\": [...]."),
        Lang::Es => format!("Un grupo de \"{event}\" no tiene \"hooks\": [...]."),
    }
}

pub fn hook_input_invalid_json(lang: Lang, title: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("\"input\" on {title} isn't valid JSON: {err}"),
        Lang::Es => format!("\"input\" de {title} no es JSON válido: {err}"),
    }
}

pub fn settings_json_invalid(lang: Lang, path: &str, err: &str) -> String {
    match lang {
        Lang::En => format!(
            "{path} already exists but has invalid JSON ({err}). Fix it by hand before saving hooks from here, so you don't lose your configuration."
        ),
        Lang::Es => format!(
            "{path} ya existe pero tiene JSON invalido ({err}). Corregilo a mano antes de guardar hooks desde aca para no perder tu configuracion."
        ),
    }
}

pub fn settings_json_not_object(lang: Lang, path: &str) -> String {
    match lang {
        Lang::En => format!("{path} isn't a valid JSON object."),
        Lang::Es => format!("{path} no es un objeto JSON valido."),
    }
}

pub fn claude_cli_not_found(lang: Lang) -> String {
    match lang {
        Lang::En => {
            "I couldn't find the Claude Code CLI (\"claude\") on your PATH, or it didn't respond to --version. \
             Install it / log in first by opening a terminal and running \"claude\" once."
                .into()
        }
        Lang::Es => {
            "No encontré el CLI de Claude Code (\"claude\") en tu PATH, o no respondió a --version. \
             Instálalo/autentícate primero abriendo una terminal y corriendo \"claude\" una vez."
                .into()
        }
    }
}

pub fn claude_exec_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't run \"claude\": {err}"),
        Lang::Es => format!("No pude ejecutar \"claude\": {err}"),
    }
}

pub fn claude_returned_error(lang: Lang, stderr: &str, code: i32) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        match lang {
            Lang::En => format!("Claude Code returned an error (exit code {code}) with no further output."),
            Lang::Es => format!("Claude Code devolvió un error (código de salida {code}) sin más detalles."),
        }
    } else {
        match lang {
            Lang::En => format!("Claude Code returned an error: {trimmed}"),
            Lang::Es => format!("Claude Code devolvió un error: {trimmed}"),
        }
    }
}

pub fn claude_command_exec_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Couldn't run the \"claude\" CLI: {err}"),
        Lang::Es => format!("No pude ejecutar el CLI \"claude\": {err}"),
    }
}

pub fn claude_response_not_json(lang: Lang, err: &str, stdout: &str) -> String {
    match lang {
        Lang::En => format!("Claude's response wasn't valid JSON: {err}\n{stdout}"),
        Lang::Es => format!("La respuesta de Claude no fue JSON válido: {err}\n{stdout}"),
    }
}

pub fn claude_response_missing_structured_output(lang: Lang, stdout: &str) -> String {
    match lang {
        Lang::En => format!("The response had no \"structured_output\": {stdout}"),
        Lang::Es => format!("La respuesta no tenía \"structured_output\": {stdout}"),
    }
}

pub fn unknown_artifact_kind(lang: Lang, kind: &str) -> String {
    match lang {
        Lang::En => format!("Unknown artifact type: {kind}"),
        Lang::Es => format!("Tipo de artefacto desconocido: {kind}"),
    }
}

// --- validate.rs ---

pub fn agent_needs_name(lang: Lang) -> String {
    match lang {
        Lang::En => "The agent needs a \"name\".".into(),
        Lang::Es => "El agente necesita un \"name\".".into(),
    }
}

pub fn name_slug_format_long(lang: Lang) -> String {
    match lang {
        Lang::En => {
            "\"name\" can only have lowercase letters, numbers and dashes (e.g. backend-java), no spaces or uppercase.".into()
        }
        Lang::Es => {
            "\"name\" solo puede tener minúsculas, números y guiones (ej: backend-java), sin espacios ni mayúsculas.".into()
        }
    }
}

pub fn name_too_long(lang: Lang, max: u32) -> String {
    match lang {
        Lang::En => format!("\"name\" can't be over {max} characters."),
        Lang::Es => format!("\"name\" no puede superar {max} caracteres."),
    }
}

pub fn agent_needs_description(lang: Lang) -> String {
    match lang {
        Lang::En => "The agent needs a \"description\": Claude Code uses it to decide when to invoke it.".into(),
        Lang::Es => "El agente necesita \"description\": Claude Code lo usa para decidir cuándo invocarlo.".into(),
    }
}

pub fn name_slug_format_short(lang: Lang) -> String {
    match lang {
        Lang::En => "\"name\" can only have lowercase letters, numbers and dashes, no spaces or uppercase.".into(),
        Lang::Es => "\"name\" solo puede tener minúsculas, números y guiones, sin espacios ni mayúsculas.".into(),
    }
}

pub fn pick_hook_event(lang: Lang) -> String {
    match lang {
        Lang::En => "Pick an event for the hook.".into(),
        Lang::Es => "Elige un evento para el hook.".into(),
    }
}

pub fn unknown_hook_event(lang: Lang, event: &str) -> String {
    match lang {
        Lang::En => format!("\"{event}\" isn't a recognized Claude Code event (check casing/spelling)."),
        Lang::Es => format!("\"{event}\" no es un evento reconocido de Claude Code (revisa mayúsculas/ortografía)."),
    }
}

pub fn http_hook_needs_url(lang: Lang) -> String {
    match lang {
        Lang::En => "An \"http\" hook needs a url.".into(),
        Lang::Es => "Un hook \"http\" necesita una url.".into(),
    }
}

pub fn mcp_tool_hook_needs_server_and_tool(lang: Lang) -> String {
    match lang {
        Lang::En => "An \"mcp_tool\" hook needs server and tool.".into(),
        Lang::Es => "Un hook \"mcp_tool\" necesita server y tool.".into(),
    }
}

pub fn prompt_hook_needs_prompt(lang: Lang) -> String {
    match lang {
        Lang::En => "A prompt/agent hook needs the prompt text.".into(),
        Lang::Es => "Un hook de tipo prompt/agent necesita el texto del prompt.".into(),
    }
}

pub fn message_action_needs_message(lang: Lang) -> String {
    match lang {
        Lang::En => "Write the message that gets added to the context.".into(),
        Lang::Es => "Escribe el mensaje que se le agrega al contexto.".into(),
    }
}

/// Título de fallback cuando un .md no tiene ni un "# Header" ni una
/// primera línea no vacía — se muestra en la lista de Rules/CLAUDE.md.
pub fn empty_placeholder(lang: Lang) -> String {
    match lang {
        Lang::En => "(empty)".into(),
        Lang::Es => "(vacío)".into(),
    }
}

pub fn command_action_needs_command(lang: Lang) -> String {
    match lang {
        Lang::En => "The hook needs a command to run.".into(),
        Lang::Es => "El hook necesita un comando a ejecutar.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_returned_error_empty_stderr_fallback() {
        let msg_en = claude_returned_error(Lang::En, "", 42);
        assert!(msg_en.contains("exit code 42"), "Expected exit code 42 in message: {msg_en}");
        assert_eq!(msg_en, "Claude Code returned an error (exit code 42) with no further output.");

        let msg_es = claude_returned_error(Lang::Es, "   \n\t", 1);
        assert!(msg_es.contains("código de salida 1"), "Expected exit code 1 in message: {msg_es}");
        assert_eq!(msg_es, "Claude Code devolvió un error (código de salida 1) sin más detalles.");

        let msg_non_empty = claude_returned_error(Lang::En, "something broke", 1);
        assert_eq!(msg_non_empty, "Claude Code returned an error: something broke");
    }
}

