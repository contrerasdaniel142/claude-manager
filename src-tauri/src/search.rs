use crate::models::{AppState, Scope};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Matches the frontend's `Category` keys so a click can `setCategory`
    /// straight away: "rules", "hooks", "skills", "agents", "outputStyles",
    /// "mcpServers".
    pub category: &'static str,
    pub id: String,
    pub title: String,
    pub snippet: String,
}

fn matches(text: &str, q: &str) -> bool {
    text.to_lowercase().contains(q)
}

fn snippet(text: &str) -> String {
    text.replace('\n', " ").chars().take(140).collect()
}

/// Simple substring search across every searchable field of every module —
/// no fuzzy scoring, no index: for the handful-to-low-hundreds of items a
/// project actually has, a plain scan of what's already on disk is fast
/// enough and has nothing to keep in sync with the data it searches.
#[tauri::command]
pub fn search_everything(state: State<AppState>, scope: Scope, query: String) -> Result<Vec<SearchResult>, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();

    for r in crate::rules::list_rules(state.clone(), scope)? {
        let full = crate::rules::read_rule(state.clone(), scope, r.id.clone())?;
        if matches(&r.title, &q) || matches(&r.id, &q) || matches(&full.body, &q) {
            out.push(SearchResult { category: "rules", id: r.id, title: r.title, snippet: snippet(&full.body) });
        }
    }

    for s in crate::skills::list_skills(state.clone(), scope)? {
        let title = if s.name.is_empty() { s.id.clone() } else { s.name.clone() };
        let full = crate::skills::read_skill(state.clone(), scope, s.id.clone())?;
        let when_to_use = full.when_to_use.as_deref().unwrap_or("");
        if matches(&title, &q) || matches(&s.description, &q) || matches(&full.body, &q) || matches(when_to_use, &q) {
            let snippet_src = if full.body.is_empty() { &s.description } else { &full.body };
            out.push(SearchResult { category: "skills", id: s.id, title, snippet: snippet(snippet_src) });
        }
    }

    for a in crate::agents::list_agents(state.clone(), scope)? {
        let title = if a.name.is_empty() { a.id.clone() } else { a.name.clone() };
        let full = crate::agents::read_agent(state.clone(), scope, a.id.clone())?;
        if matches(&title, &q) || matches(&a.description, &q) || matches(&full.body, &q) {
            let snippet_src = if full.body.is_empty() { &a.description } else { &full.body };
            out.push(SearchResult { category: "agents", id: a.id, title, snippet: snippet(snippet_src) });
        }
    }

    for o in crate::output_styles::list_output_styles(state.clone(), scope)? {
        let full = crate::output_styles::read_output_style(state.clone(), scope, o.id.clone())?;
        if matches(&o.name, &q) || matches(&o.description, &q) || matches(&full.body, &q) {
            let snippet_src = if full.body.is_empty() { &o.description } else { &full.body };
            out.push(SearchResult { category: "outputStyles", id: o.id, title: o.name, snippet: snippet(snippet_src) });
        }
    }

    for m in crate::mcp_servers::list_mcp_servers(state.clone(), scope)? {
        if matches(&m.name, &q) || matches(&m.server_type, &q) {
            out.push(SearchResult {
                category: "mcpServers",
                id: m.name.clone(),
                title: m.name,
                snippet: m.server_type,
            });
        }
    }

    let hooks_doc = crate::hooks::read_hooks(state, scope)?;
    for (i, h) in hooks_doc.rules.iter().enumerate() {
        let haystacks = [h.title.as_str(), h.description.as_str(), h.command.as_str(), h.message.as_str(), h.event.as_str()];
        if haystacks.iter().any(|t| matches(t, &q)) {
            let title = if h.title.is_empty() { h.event.clone() } else { h.title.clone() };
            let body = if !h.description.is_empty() {
                &h.description
            } else if !h.message.is_empty() {
                &h.message
            } else {
                &h.command
            };
            out.push(SearchResult { category: "hooks", id: i.to_string(), title, snippet: snippet(body) });
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_is_case_insensitive() {
        assert!(matches("Clean Architecture", "architecture"));
        assert!(!matches("Clean Architecture", "hexagonal"));
    }

    #[test]
    fn snippet_collapses_newlines_and_truncates_to_140_chars() {
        let text = format!("line one\nline two\n{}", "x".repeat(200));
        let s = snippet(&text);
        assert!(!s.contains('\n'));
        assert_eq!(s.chars().count(), 140);
    }
}
