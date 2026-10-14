# claude-manager

Gestor visual de docs de steering, hooks, agentes y skills de Claude Code. Tauri 2 + React 19 + TypeScript. Frontend en `src/`, backend Rust en `src-tauri/`.

## Skill obligatoria

- Usá SIEMPRE la skill `ponytail` en cualquier tarea de código: escribir, arreglar, refactorizar, revisar o elegir dependencias.
- Invocala con la herramienta Skill antes de tocar código, no después.
- Usá el nivel por defecto (full) salvo que el usuario pida `lite` o `ultra`.

## Stack

- Frontend: React 19, TypeScript ~6, Vite 8, Tailwind 4 (`@tailwindcss/vite`), react-markdown.
- Backend: Rust edition 2024, Tauri 2, crate `clman` (lib `claude_manager_lib`).
- Plugins de Tauri: `tauri-plugin-opener`, `tauri-plugin-dialog`.
- Crates clave: serde / serde_json / serde_yaml, notify 6 (watcher), walkdir, regex, tempfile, dirs 5.

## Estructura

- `src/`: UI React + TypeScript.
- `src-tauri/src/`: módulos de backend. Dominio: `agents.rs`, `skills.rs`, `hooks.rs`, `rules.rs`, `steering.rs`, `permissions.rs`, `mcp_servers.rs`, `plugins.rs`, `output_styles.rs`. Importación: `skill_import.rs`, `mcp_import.rs`. Infra: `claude_cli.rs`, `git.rs`, `watcher.rs`, `fsutil.rs`, `validate.rs`, `i18n.rs`, `assistant.rs`, `models.rs`.
- `src-tauri/src/main.rs` y `lib.rs`: arranque y registro de comandos Tauri.
- `src-tauri/tauri.conf.json` y `src-tauri/Cargo.toml`: config de la app y dependencias Rust.

## Comandos

- Instalar dependencias: `npm install`
- Frontend en modo dev: `npm run dev`
- App completa en modo dev: `npm run tauri dev`
- Typecheck + build del frontend: `npm run build`
- Build de la app: `npm run tauri build`
- Verificar Rust sin compilar binarios: `cargo check --manifest-path src-tauri/Cargo.toml`

Después de cambiar TypeScript corré `npm run build`. Después de cambiar Rust corré `cargo check`. Reportá la salida si falla.

## Convenciones

- Cambios mínimos: hacé solo lo que la tarea pide, sin refactors ni archivos extra no pedidos.
- Antes de agregar texto visible para el usuario, revisá `src-tauri/src/i18n.rs` y seguí el mismo patrón.
- Las escrituras a archivos de configuración de Claude Code (`~/.claude/`, `.claude/` del proyecto) pasan por `fsutil.rs` y validan con `validate.rs` antes de escribir.
- Los errores de backend se devuelven como `Result` a la UI; no uses `unwrap()` ni `expect()` en código que maneja archivos o entrada del usuario.
- Los cambios de estado del lado Rust que afectan la UI deben emitirse o reflejarse en el watcher (`watcher.rs`), no con polling nuevo.

## Reglas duras

- No borres ni sobrescribas archivos de `.claude/` del usuario sin confirmación explícita.
- No edites `target/` ni `node_modules/`.
- No agregues dependencias nuevas sin justificarlas en una línea; ponytail aplica.
- No hagas commit ni push sin que el usuario lo pida.
- No desactives hooks, permisos ni validaciones para que un test o build pase.
