use crate::fsutil::silent_command;
use crate::i18n;
use crate::models::AppState;
use crate::validate::KNOWN_HOOK_EVENTS;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tauri::State;

static CLAUDE_BIN: OnceLock<Option<String>> = OnceLock::new();

/// Finds a working `claude` CLI on this machine. None if it's not on PATH or
/// doesn't respond to --version — same "verify it actually works, not just
/// that it spawned" lesson learned from detect_python().
fn detect_claude() -> Option<&'static str> {
    CLAUDE_BIN
        .get_or_init(|| {
            let output = silent_command("claude").arg("--version").output().ok()?;
            if output.status.success() {
                Some("claude".to_string())
            } else {
                None
            }
        })
        .as_deref()
}

#[tauri::command]
pub fn assistant_available() -> bool {
    detect_claude().is_some()
}

fn schema_for(lang: crate::models::Lang, kind: &str) -> Result<Value, String> {
    Ok(match kind {
        "rule" => json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Nombre de archivo sin .md, minúsculas y guiones, ej: clean-architecture-java"},
                "paths": {"type": "array", "items": {"type": "string"}, "description": "Patrones glob (ej '**/*.java'). Array vacío = se carga siempre. SIEMPRE devolvé este campo, aunque sea un array vacío."},
                "body": {"type": "string", "description": "SOLO el contenido markdown (headers y bullets, sin front matter). NUNCA incluyas '---' ni 'paths:' acá adentro: eso va aparte, en el campo 'paths'."}
            },
            "required": ["id", "paths", "body"]
        }),
        "advisor" => json!({
            "type": "object",
            "properties": {
                "candidates": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 3,
                    "items": {
                        "type": "object",
                        "properties": {
                            "kind": {"type": "string", "enum": ["rule", "hook", "skill", "outputStyle", "permissions", "mcpServer", "plugin"]},
                            "reason": {"type": "string", "description": "1-2 oraciones explicando por qué esta herramienta encaja con el pedido específico (no una definición genérica de la herramienta)."}
                        },
                        "required": ["kind", "reason"]
                    },
                    "description": "Normalmente un solo candidato. Hasta 3 solo si el pedido genuinamente podría resolverse con más de una herramienta, ordenados del más al menos probable."
                }
            },
            "required": ["candidates"]
        }),
        "mcpServer" => json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Nombre corto del servidor, minúsculas y guiones, ej: github, database-tools."},
                "serverType": {"type": "string", "enum": ["stdio", "http", "sse"], "description": "'stdio' corre un proceso local (command+args). 'http' (preferido) o 'sse' (deprecado) se conectan a una URL remota."},
                "command": {"type": ["string", "null"], "description": "Requerido si serverType es 'stdio': el ejecutable a lanzar, ej 'npx'."},
                "args": {"type": "array", "items": {"type": "string"}, "description": "Solo 'stdio': argumentos de línea de comandos. Array vacío si no aplica."},
                "env": {"type": ["string", "null"], "description": "Solo 'stdio': variables de entorno para el proceso, una por línea como 'CLAVE=valor'. null si no hace falta."},
                "url": {"type": ["string", "null"], "description": "Requerido si serverType es 'http' o 'sse'."},
                "headers": {"type": ["string", "null"], "description": "Solo 'http'/'sse': headers HTTP, uno por línea como 'Clave: Valor'. null si no hace falta."}
            },
            "required": ["name", "serverType", "args"]
        }),
        "permissions" => json!({
            "type": "object",
            "properties": {
                "allow": {"type": "array", "items": {"type": "string"}, "description": "Reglas de permiso que corren sin preguntar. Formato 'Tool' o 'Tool(especificador)', ej 'Bash(npm run test)', 'Read(~/.zshrc)'. Array vacío si no aplica."},
                "ask": {"type": "array", "items": {"type": "string"}, "description": "Reglas que siempre preguntan antes de correr, aunque una regla allow más específica también matchee."},
                "deny": {"type": "array", "items": {"type": "string"}, "description": "Reglas bloqueadas siempre, sin excepción (gana sobre allow/ask). Ej 'Bash(curl *)', 'Read(./.env)', 'Read(./secrets/**)'."},
                "additionalDirectories": {"type": "array", "items": {"type": "string"}, "description": "Directorios fuera del working directory a los que Claude puede acceder. Array vacío si no aplica."},
                "defaultMode": {"type": ["string", "null"], "enum": [null, "default", "acceptEdits", "plan", "auto", "dontAsk", "bypassPermissions"], "description": "Modo de permisos con el que arranca la sesión. null = no tocar (usa el default de Claude Code)."},
                "disableBypassPermissionsMode": {"type": "boolean", "description": "true bloquea que la sesión pueda usar bypassPermissions."},
                "disableAutoMode": {"type": "boolean", "description": "true bloquea que la sesión pueda usar auto mode."},
                "blockReadsOutsideWorkingDirectories": {"type": "boolean", "description": "true impide leer archivos fuera de los working directories en cualquier modo."},
                "attributionCommit": {"type": ["string", "null"], "description": "Texto del trailer que Claude Code agrega a los commits. null = no tocar (deja el default 'Co-Authored-By: Claude'). '' (string vacío) = ocultarlo del todo."},
                "attributionPr": {"type": ["string", "null"], "description": "Texto de atribución en la descripción de los PRs. null = no tocar (deja el default). '' = ocultarlo del todo."},
                "attributionSessionUrl": {"type": ["boolean", "null"], "description": "false oculta el link a la sesión de claude.ai en commits/PRs hechos en la nube. null = no tocar (default true)."},
                "includeGitInstructions": {"type": ["boolean", "null"], "description": "false le saca a Claude las instrucciones propias de Claude Code sobre cómo hacer commits/PRs. null = no tocar (default true)."}
            },
            "required": ["allow", "ask", "deny", "additionalDirectories", "disableBypassPermissionsMode", "disableAutoMode", "blockReadsOutsideWorkingDirectories"]
        }),
        "outputStyle" => json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Nombre de archivo sin .md, minúsculas y guiones, ej: diagrams-first"},
                "name": {"type": ["string", "null"], "description": "Nombre lindo mostrado en el picker de /output-style. null = usa el nombre de archivo."},
                "description": {"type": ["string", "null"], "description": "Descripción corta mostrada en el picker. null si no hace falta."},
                "keepCodingInstructions": {"type": "boolean", "description": "true si Claude sigue codeando igual (solo cambia tono/formato); false si el estilo lo convierte en otra cosa (ej un asistente de escritura) y no necesita las instrucciones de ingeniería de software."},
                "body": {"type": "string", "description": "SOLO las instrucciones en markdown (sin front matter, sin '---')."}
            },
            "required": ["id", "keepCodingInstructions", "body"]
        }),
        "hook" => json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Nombre corto para identificar el hook en la lista."},
                "description": {"type": "string"},
                "event": {"type": "string", "enum": KNOWN_HOOK_EVENTS},
                "matcher": {"type": "array", "items": {"type": "string"}, "description": "Según el evento: nombres de herramienta, nombres de archivo literales, o palabra clave. Array vacío = aplica siempre."},
                "hookType": {"type": "string", "enum": ["command", "http", "mcp_tool", "prompt", "agent"], "description": "Tipo real de Claude Code. 'command' para la gran mayoría de los casos. Default: 'command'."},
                "ifFilter": {"type": ["string", "null"], "description": "Filtra con sintaxis de permission rule (ej 'Bash(git *)'), solo se evalúa en eventos de tool. Solo si hace falta, si no null."},
                "statusMessage": {"type": ["string", "null"], "description": "Mensaje de spinner mientras corre, null si no hace falta."},
                "fileGlob": {"type": "array", "items": {"type": "string"}, "description": "Solo con hookType 'command' y sin wrapper avanzado: filtra por extensión de archivo (ej '*.java')."},
                "action": {"type": "string", "enum": ["command", "message"], "description": "Solo aplica con hookType 'command'."},
                "command": {"type": "string", "description": "Comando de shell, solo si hookType es 'command' y action es 'command'."},
                "message": {"type": "string", "description": "Texto a agregarle al contexto de Claude, solo si hookType es 'command' y action es 'message'."},
                "args": {"type": ["array", "null"], "items": {"type": "string"}, "description": "Solo hookType 'command': argv explícito, corre el ejecutable sin shell. null si no hace falta."},
                "shell": {"type": ["string", "null"], "enum": [null, "bash", "powershell"], "description": "Solo hookType 'command'. null = default (bash, o powershell en Windows sin Git Bash)."},
                "onFailure": {"type": ["string", "null"], "enum": [null, "continue", "block"], "description": "hookType 'command' o 'http'. null = 'continue' (default)."},
                "asyncRewake": {"type": ["boolean", "null"], "description": "Solo hookType 'command' con isAsync true: despierta a Claude si el comando sale con código 2."},
                "timeout": {"type": ["integer", "null"], "description": "Segundos, null si no hace falta un valor especial."},
                "isAsync": {"type": "boolean", "description": "Solo tiene sentido con hookType 'command'."},
                "url": {"type": ["string", "null"], "description": "Requerido si hookType es 'http'."},
                "headers": {"type": ["string", "null"], "description": "Solo hookType 'http': una línea 'Clave: Valor' por header, null si no hace falta."},
                "allowedEnvVars": {"type": ["array", "null"], "items": {"type": "string"}, "description": "Solo hookType 'http': variables de entorno interpolables en headers."},
                "server": {"type": ["string", "null"], "description": "Requerido si hookType es 'mcp_tool': nombre del servidor MCP configurado."},
                "tool": {"type": ["string", "null"], "description": "Requerido si hookType es 'mcp_tool': nombre de la tool a llamar en ese servidor."},
                "input": {"type": ["string", "null"], "description": "Solo hookType 'mcp_tool': JSON con los argumentos para la tool, como texto. Soporta '${tool_input.file_path}' etc."},
                "prompt": {"type": ["string", "null"], "description": "Requerido si hookType es 'prompt' o 'agent'. Usá '$ARGUMENTS' para el JSON de entrada del hook."},
                "promptModel": {"type": ["string", "null"], "description": "Solo hookType 'prompt'/'agent'. null = usa el modelo default de background."}
            },
            "required": ["event", "hookType"]
        }),
        "agent" => json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "minúsculas y guiones, ej: backend-java"},
                "description": {"type": "string", "description": "Cuándo Claude Code debería delegarle tareas a este agente."},
                "tools": {"type": "array", "items": {"type": "string"}, "description": "Array vacío = hereda todas."},
                "model": {"type": "string", "enum": ["", "inherit", "sonnet", "opus", "haiku", "fable"]},
                "disallowedTools": {"type": "array", "items": {"type": "string"}},
                "permissionMode": {"type": "string", "enum": ["", "default", "acceptEdits", "auto", "dontAsk", "bypassPermissions", "plan", "manual"]},
                "color": {"type": "string", "enum": ["", "red", "blue", "green", "yellow", "purple", "orange", "pink", "cyan"]},
                "maxTurns": {"type": ["integer", "null"]},
                "skills": {"type": "array", "items": {"type": "string"}, "description": "Solo nombres de skills que ya existan; si no sabés cuáles existen, dejalo vacío."},
                "memory": {"type": ["string", "null"], "enum": [null, "user", "project", "local"], "description": "Memoria persistente propia del agente, entre sesiones. null = sin memoria propia."},
                "background": {"type": ["boolean", "null"], "description": "true = sigue en background aunque Claude pida foreground. null si no hace falta definirlo."},
                "omitClaudeMd": {"type": ["boolean", "null"], "description": "true = no cargar los CLAUDE.md del proyecto para este agente. null = default (los carga)."},
                "effort": {"type": ["string", "null"], "enum": [null, "low", "medium", "high", "xhigh", "max"]},
                "isolation": {"type": ["string", "null"], "enum": [null, "worktree"], "description": "'worktree' corre al agente en un git worktree temporal aislado."},
                "initialPrompt": {"type": ["string", "null"], "description": "Primer turno auto-enviado si este agente corre como agente principal (--agent). null si no aplica."},
                "cacheTtl": {"type": ["string", "null"], "enum": [null, "5m", "1h"], "description": "experimental.cacheTtl — tiempo de vida del cache de prompt. null si no hace falta."},
                "mcpServersRaw": {"type": ["string", "null"], "description": "JSON crudo: lista de nombres de servidores MCP ya configurados, o configs inline. null si no hace falta."},
                "hooksRaw": {"type": ["string", "null"], "description": "JSON crudo: hooks propios del agente, mismo formato que settings.json. null si no hace falta."},
                "body": {"type": "string", "description": "El system prompt completo del agente."}
            },
            "required": ["name", "description", "body"]
        }),
        "skill" => json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "minúsculas y guiones, máx 64 caracteres. Vacío = usa el nombre de carpeta."},
                "description": {"type": "string"},
                "whenToUse": {"type": "string"},
                "allowedTools": {"type": "array", "items": {"type": "string"}},
                "paths": {"type": "array", "items": {"type": "string"}, "description": "Patrones glob — activación automática solo cuando Claude toca estos archivos. Array vacío = sin restricción."},
                "argumentHint": {"type": ["string", "null"], "description": "Hint de autocompletado, ej '[issue-number]'. null si no hace falta."},
                "arguments": {"type": ["string", "null"], "description": "Nombres de argumentos posicionales separados por espacio, ej 'issue-number'. null si no hace falta."},
                "disableModelInvocation": {"type": ["boolean", "null"], "description": "true = Claude no lo invoca solo, solo corre a mano con /nombre."},
                "userInvocable": {"type": ["boolean", "null"], "description": "false = no aparece en el menú ni /nombre lo corre, solo Claude puede invocarlo."},
                "disallowedTools": {"type": "array", "items": {"type": "string"}, "description": "Herramientas que se le sacan a Claude mientras este skill está activo."},
                "model": {"type": ["string", "null"], "enum": [null, "inherit", "sonnet", "opus", "haiku", "fable"]},
                "effort": {"type": ["string", "null"], "enum": [null, "low", "medium", "high", "xhigh", "max"]},
                "context": {"type": ["string", "null"], "enum": [null, "fork"], "description": "'fork' corre el skill en un subagente aparte."},
                "agent": {"type": ["string", "null"], "description": "Solo con context 'fork': qué subagente usar (Explore, Plan, general-purpose, o uno custom). null si context no es 'fork'."},
                "background": {"type": ["boolean", "null"], "description": "Solo con context 'fork'. false = espera el resultado del subagente en el mismo turno."},
                "shell": {"type": ["string", "null"], "enum": [null, "bash", "powershell"]},
                "license": {"type": ["string", "null"]},
                "compatibility": {"type": ["string", "null"], "description": "Requisitos de entorno, máx 500 caracteres."},
                "hooksRaw": {"type": ["string", "null"], "description": "JSON crudo: hooks propios del skill, mismo formato que settings.json. null si no hace falta."},
                "metadataRaw": {"type": ["string", "null"], "description": "JSON crudo: datos libres, Claude Code no los usa. null si no hace falta."},
                "body": {"type": "string", "description": "Contenido completo de SKILL.md."}
            },
            "required": ["description", "body"]
        }),
        "claudemd" => json!({
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "Contenido completo del CLAUDE.md, markdown, sin front matter."}
            },
            "required": ["content"]
        }),
        other => return Err(i18n::unknown_artifact_kind(lang, other)),
    })
}

fn briefing_for(kind: &str) -> &'static str {
    match kind {
        "advisor" => "Tenés que decidir qué herramienta(s) de Claude Code resuelven mejor lo que describe el usuario, entre estas 7: \
'rule' (.claude/rules/*.md): instrucciones de código/arquitectura que se cargan solas cuando Claude toca ciertos archivos (o siempre, sin paths) — convenciones y reglas duras de un proyecto. \
'hook' (.claude/settings.json): algo que tiene que pasar SÍ O SÍ ante un evento puntual de Claude Code (antes/después de usar una herramienta, al terminar la sesión, etc.) — no depende de que Claude \"decida\" seguirlo. \
'skill' (.claude/skills/*/SKILL.md): instrucciones para una tarea específica que Claude invoca cuando la tarea coincide, o que corre como comando /nombre. \
'outputStyle' (.claude/output-styles/*.md): cambia el tono/formato/rol de Claude para TODA la sesión (ej \"que sea más conciso\", \"que explique sus decisiones\", \"que sea un asistente de escritura\") — no para una tarea puntual. \
'permissions' (claves \"permissions\"/\"attribution\" de settings.json): reglas de qué puede hacer Claude sin preguntar, qué tiene que confirmar siempre, o qué está bloqueado (allow/ask/deny de herramientas o comandos) — también incluye atribución en git (ej. \"que no agregue Co-Authored-By a los commits\", \"sacar la mención a Claude de los PRs\"): eso es un ajuste de settings, NUNCA un hook (un hook PreToolUse llega tarde para cambiar el commit que Claude ya está corriendo). \
'mcpServer': conectar una herramienta o servicio externo (base de datos, API, servicio remoto) como servidor MCP para darle tools nuevas a Claude. \
'plugin': un paquete ya armado por otra persona/equipo, instalable desde un marketplace — solo aplica si el pedido es \"instalar algo que ya existe\" en vez de \"crear algo específico para mi caso\"; si hay duda entre un plugin y crear la herramienta puntual uno mismo, preferí la herramienta puntual. \
Devolvé candidatos ordenados del más al menos probable — normalmente uno solo; varios solo si el pedido genuinamente admite más de una solución razonable, explicando en \"reason\" por qué ESE pedido puntual encaja con esa herramienta, no una definición genérica.",
        "mcpServer" => "Estás ayudando a configurar un servidor MCP para Claude Code. 'stdio' es para un servidor que corre como proceso local (necesita 'command', opcionalmente 'args' y 'env') — es el tipo más común para herramientas que corren en la máquina del usuario. 'http' (preferido sobre 'sse', que está deprecado) es para un servidor remoto, accesible por URL — necesita 'url' y opcionalmente 'headers' para autenticación (ej 'Authorization: Bearer $TOKEN'). No inventes un 'command' con curl para conectarse a una URL remota: si el pedido menciona una URL o un servidor remoto, usa 'http'.",
        "permissions" => "Estás ayudando a configurar los permisos de Claude Code (clave \"permissions\" en settings.json) para este alcance. Las reglas son strings con formato 'Tool' o 'Tool(especificador)' — ej 'Bash(npm run test)', 'Bash(git push *)' (el '*' es wildcard), 'Read(~/.zshrc)', 'Read(./secrets/**)' (gitignore-style), 'WebFetch(domain:example.com)', 'mcp__servidor__tool'. Orden de evaluación: deny gana siempre, después ask, después allow — por eso los secretos van en \"deny\", no alcanza con no ponerlos en \"allow\". \"ask\" es para acciones que SIEMPRE tienen que confirmarse aunque una regla allow más específica también matchee (ej. 'Bash(git push *)' en ask, aunque 'Bash(git *)' esté en allow). No completes defaultMode ni los boolean flags salvo que el pedido lo mencione explícitamente — la mayoría de los pedidos son \"permitir X\", \"bloquear Y\", \"preguntar siempre por Z\", que van directo a allow/deny/ask. Para pedidos sobre atribución en git (ej. \"que no agregue Co-Authored-By a los commits\", \"sacar la mención a Claude de los PRs\") usá attributionCommit/attributionPr en '' (string vacío), NO un hook — un hook PreToolUse con additionalContext llega tarde para cambiar el commit que Claude ya está corriendo, estos campos lo evitan de raíz.",
        "outputStyle" => "Estás ayudando a crear un Output Style de Claude Code (.claude/output-styles/*.md). Cambia el rol/tono/formato de Claude para toda la sesión, no reemplaza CLAUDE.md (conocimiento del proyecto), un Skill (instrucciones para una tarea puntual) ni un Hook (algo que tiene que pasar sí o sí). \"keepCodingInstructions\" en true mantiene las instrucciones de ingeniería de software de Claude Code de fondo (usalo cuando el pedido es solo sobre CÓMO responde, ej. \"que sea más conciso\", \"que explique sus decisiones\"); en false cuando el pedido convierte a Claude en otra cosa (ej. \"asistente de escritura\", \"analista de datos\") que no necesita esas instrucciones.",
        "rule" => "Estás ayudando a crear una Rule de Claude Code (.claude/rules/*.md). Sin \"paths\", se carga siempre; con \"paths\", solo cuando Claude toca un archivo que coincide con esos patrones glob. Si ayuda a que el contenido sea específico del proyecto real (nombres de paquetes, estructura de carpetas existente) en vez de genérico, podés leer el código con las herramientas disponibles antes de responder.",
        "hook" => "Estás ayudando a crear un Hook de Claude Code (.claude/settings.json). IMPORTANTE sobre \"hookType\": existen 5 tipos NATIVOS de Claude Code y tenés que elegir el que corresponde de verdad, no reimplementarlo a mano con \"command\": si el pedido es \"avisar/mandar/notificar a una URL o webhook\" usá hookType \"http\" con el campo \"url\" — NUNCA generes un \"command\" con curl/Invoke-WebRequest para esto, aunque funcione, no es lo que se pidió y le faltan los campos reales (url, headers, allowedEnvVars) que el usuario después no puede editar como tales. Si el pedido es \"llamar una tool de un servidor MCP\" usá hookType \"mcp_tool\" con \"server\"+\"tool\". Si el pedido es \"preguntarle a un modelo\"/\"evaluar con IA si algo es seguro\" usá hookType \"prompt\" o \"agent\" con el campo \"prompt\". Recién si ninguno de esos aplica (correr un script, un linter, un comando real del sistema) usá hookType \"command\". Dentro de \"command\": \"action\": \"message\" agrega el texto al contexto real de Claude (vía hookSpecificOutput.additionalContext — así lo resuelve clman, no hace falta que te preocupes por el JSON); \"command\" corre un comando de shell de verdad. Para que algo le \"llegue\" a Claude solo hay dos caminos: \"message\", o un \"command\" cuyo script imprima él mismo ese JSON con additionalContext — un comando que solo lee/reescribe archivos (ej. volver a guardar un archivo con el mismo contenido) NO le comunica nada a Claude, es un efecto invisible para el modelo. No inventes ese tipo de workaround. Dato específico: una Rule con \"paths\" solo se carga sola cuando Claude hace Read de un archivo que matchea — NO cuando Claude recién lo crea con Write (bug conocido de Claude Code, github.com/anthropics/claude-code/issues/63142). Si te piden un hook para ese caso puntual, usá hookType \"command\", \"action\": \"message\" con evento PostToolUse y matcher \"Write\", explicando en el mensaje qué regla aplicarle — no intentes \"forzar\" la carga de la regla reescribiendo el archivo. NUNCA llames a herramientas como Write/Bash/Skill para intentar guardar el resultado vos mismo en settings.json — tu única salida es el JSON estructurado, la app es la que escribe el archivo.",
        "agent" => "Estás ayudando a crear un subagente de Claude Code (.claude/agents/*.md). Claude delega tareas a este agente automáticamente cuando el pedido coincide con su \"description\". Si ayuda, podés leer el código del proyecto antes de escribir el system prompt para que sea específico, no genérico. Hay varios campos avanzados (memory, background, omitClaudeMd, effort, isolation, initialPrompt, cacheTtl, mcpServersRaw, hooksRaw) — dejalos en null salvo que el pedido del usuario mencione explícitamente algo que los necesite (ej. \"que recuerde cosas entre sesiones\" → memory, \"que no cargue el CLAUDE.md\" → omitClaudeMd). No los completes \"porque existen\".",
        "skill" => "Estás ayudando a crear un Skill de Claude Code (.claude/skills/*/SKILL.md). Claude lo invoca solo cuando la tarea coincide con su \"description\"/\"when_to_use\". Si ayuda, podés leer el código del proyecto antes de escribir el contenido para que sea específico, no genérico. Hay varios campos avanzados (paths, argumentHint, arguments, disableModelInvocation, userInvocable, context/agent, hooksRaw, metadataRaw, license, compatibility) — dejalos vacíos/null salvo que el pedido los mencione explícitamente (ej. \"que se active solo con archivos .ts\" → paths, \"que corra como comando de barra\" → userInvocable/disableModelInvocation). No los completes \"porque existen\".",
        "claudemd" => "Estás ayudando a crear o editar un CLAUDE.md de Claude Code — instrucciones persistentes para el proyecto (comandos, stack, convenciones, reglas duras). Si el campo \"content\" ya tiene texto, es una edición: agregá o modificá solo lo que pide el usuario, conservando el resto tal cual. Si \"content\" está vacío, es una creación nueva: antes de escribir, USÁ LAS HERRAMIENTAS (Read, Glob, Grep) para explorar el proyecto de verdad — mirá package.json/pom.xml/requirements.txt/go.mod si existen, el árbol de carpetas, y el README — y basá el contenido en lo que encontrás, igual que hace /init, no en suposiciones genéricas. Seguí las buenas prácticas: instrucciones concretas e imperativas, nada de consejos genéricos tipo 'escribí código limpio', bajo ~200-300 líneas.",
        _ => "",
    }
}

pub const ASSISTANT_MODELS: &[&str] = &["haiku", "sonnet", "opus", "fable"];

#[tauri::command]
pub fn list_assistant_models() -> Vec<&'static str> {
    ASSISTANT_MODELS.to_vec()
}

#[tauri::command]
pub async fn generate_with_ai(
    state: State<'_, AppState>,
    kind: String,
    prompt: String,
    current_fields: Value,
    model: Option<String>,
) -> Result<Value, String> {
    // `claude -p` takes several seconds (confirmed: 9-18s depending on how
    // much it explores). Running this as a plain sync command would block
    // Tauri's command dispatch for anything else invoked while it's in
    // flight, same issue as skill_import's git clones.
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || generate_with_ai_impl(lang, kind, prompt, current_fields, model))
        .await
        .map_err(|e| e.to_string())?
}

fn generate_with_ai_impl(
    lang: crate::models::Lang,
    kind: String,
    prompt: String,
    current_fields: Value,
    model: Option<String>,
) -> Result<Value, String> {
    let claude = detect_claude().ok_or_else(|| i18n::claude_cli_not_found(lang))?;

    let schema = schema_for(lang, &kind)?;
    let briefing = briefing_for(&kind);
    let model = model.filter(|m| !m.trim().is_empty()).unwrap_or_else(|| "haiku".to_string());

    let editing_instructions = if kind == "advisor" {
        ""
    } else {
        editing_instructions_for(&current_fields)
    };

    let full_prompt = format!(
        "{briefing}\n\n{editing_instructions}\n\nLo que ya está completado en el formulario (puede estar vacío):\n{}\n\nPedido del usuario: {prompt}\n\nNunca escribas '---' ni un bloque de front matter adentro de un campo de texto libre (body/command/message): los metadatos van siempre en su propio campo del JSON, nunca mezclados con el contenido. Respondé solo con los datos, sin explicación.",
        serde_json::to_string_pretty(&current_fields).unwrap_or_default(),
    );
    let schema_str = serde_json::to_string(&schema).map_err(|e| e.to_string())?;

    let output = silent_command(claude)
        .arg("-p")
        .arg(&full_prompt)
        .arg("--model")
        .arg(&model)
        // Solo lectura: puede mirar el código real para no inventar nombres
        // de archivo, convenciones o estructura, pero nunca escribe nada —
        // eso lo hace esta misma función al volcar structured_output en el
        // formulario, y el usuario recién al apretar Guardar.
        .arg("--allowedTools")
        .arg("Read,Glob,Grep")
        .arg("--output-format")
        .arg("json")
        .arg("--json-schema")
        .arg(&schema_str)
        .arg("--permission-prompts")
        .arg("none")
        .output()
        .map_err(|e| i18n::claude_exec_failed(lang, &e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(i18n::claude_returned_error(lang, stderr.trim(), code));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)
        .map_err(|e| i18n::claude_response_not_json(lang, &e.to_string(), &stdout))?;

    parsed
        .get("structured_output")
        .cloned()
        .ok_or_else(|| i18n::claude_response_missing_structured_output(lang, &stdout))
}

fn editing_instructions_for(current_fields: &Value) -> &'static str {
    let has_existing_content = !current_fields.is_null()
        && current_fields
            .as_object()
            .map(|o| o.values().any(|v| !v.is_null() && v != "" && v != &Value::Array(vec![])))
            .unwrap_or(false);
    let editing_instructions = if has_existing_content {
        "Esto YA TIENE contenido (ver más abajo) — es una edición, no una creación desde cero. \
         Tratalo como un editor preciso, no como un generador: \
         (1) Si el pedido del usuario apunta a algo puntual (\"arreglá X\", \"agregá Y\", \"el nombre está mal\", \"cambiá el evento a Z\"), modificá SOLO ese campo o esa parte puntual del texto. \
         (2) Todo campo que el pedido no menciona va EXACTAMENTE IGUAL a como está en el formulario actual — copialo literal, no lo reescribas, no lo \"mejores\", no lo resumas ni le cambies el estilo aunque te parezca que podrías redactarlo mejor. \
         (3) Si el pedido es genérico (\"generá algo para X\") sin mencionar partes puntuales, ahí sí podés completar o rehacer lo que haga falta."
    } else {
        "Esto se está creando desde cero (el formulario está vacío) — completá todos los campos relevantes según el pedido del usuario."
    };
    editing_instructions
}
