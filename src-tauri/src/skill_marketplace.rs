//! Discovery for skills.sh (vercel-labs/skills) — the same community-wide
//! skill index its own official `skills` CLI uses for `skills find`. We hit
//! its `/api/search` endpoint directly (same one the CLI calls — confirmed
//! by inspecting the published package) instead of that CLI, since the CLI's
//! own install flow assumes a node_modules/package.json project, which would
//! force an npm dependency onto projects clman manages that aren't Node at
//! all. Install reuses the existing git-clone-and-copy path from
//! skill_import.rs instead.
use crate::git::clone_repo;
use crate::i18n;
use crate::models::{AppState, Lang, Scope};
use crate::skill_import::{find_skill_rel_path, install_from_tmp};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketplaceSkillSummary {
    /// "owner/repo" on GitHub — what we clone to install it.
    pub source: String,
    /// The skill's folder name inside that repo, also its install id.
    pub skill_id: String,
    pub name: String,
    pub installs: u64,
}

fn parse_search_response(body: &str) -> Vec<MarketplaceSkillSummary> {
    let doc: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    doc.get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| MarketplaceSkillSummary {
                    source: v.get("source").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    skill_id: v.get("skillId").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    name: v.get("name").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                    installs: v.get("installs").and_then(|x| x.as_u64()).unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// skills.sh itself rejects anything under 2 chars — mirrored here so we
/// don't even make the request for a query that will just error.
fn search_marketplace_skills_impl(lang: Lang, query: &str) -> Result<Vec<MarketplaceSkillSummary>, String> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }
    let body = ureq::get("https://skills.sh/api/search")
        .query("q", query)
        .call()
        .map_err(|e| i18n::marketplace_request_failed(lang, &e.to_string()))?
        .into_string()
        .map_err(|e| i18n::marketplace_request_failed(lang, &e.to_string()))?;
    Ok(parse_search_response(&body))
}

#[tauri::command]
pub async fn search_marketplace_skills(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<MarketplaceSkillSummary>, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || search_marketplace_skills_impl(lang, &query))
        .await
        .map_err(|e| e.to_string())?
}

fn install_marketplace_skill_impl(
    state: &AppState,
    source: String,
    skill_id: String,
    scope: Scope,
) -> Result<String, String> {
    let lang = state.lang();
    let repo_url = format!("https://github.com/{source}");
    let tmp = clone_repo(lang, &repo_url)?;
    let result = (|| {
        let rel_path =
            find_skill_rel_path(&tmp, &skill_id).ok_or_else(|| i18n::skill_not_found_in_repo(lang, &skill_id))?;
        install_from_tmp(state, lang, &tmp, &rel_path, scope)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SecurityAudit {
    pub name: String,
    pub verdict: String,
}

fn extract_security_audits(html: &str) -> Vec<SecurityAudit> {
    let marker = "Security Audits";
    let marker_pos = match html.find(marker) {
        Some(pos) => pos + marker.len(),
        None => return Vec::new(),
    };
    let mut rest = &html[marker_pos..];

    let mut audits = Vec::new();
    while let Some(a_pos) = rest.find("<a ") {
        rest = &rest[a_pos..];
        let a_end = match rest.find("</a>") {
            Some(pos) => pos,
            None => break,
        };
        let a_block = &rest[..a_end];
        rest = &rest[a_end + 4..];

        let name_marker = "text-sm font-medium";
        let name_idx = match a_block.find(name_marker) {
            Some(idx) => idx + name_marker.len(),
            None => continue,
        };
        let after_name_marker = &a_block[name_idx..];
        let name_start = match after_name_marker.find('>') {
            Some(idx) => idx + 1,
            None => continue,
        };
        let name_content = &after_name_marker[name_start..];
        let name_end = match name_content.find("</span>") {
            Some(idx) => idx,
            None => continue,
        };
        let name = name_content[..name_end].trim().to_string();

        let after_name_span = &after_name_marker[name_start + name_end + 7..];
        let verdict_marker = "text-xs font-mono uppercase";
        let verdict_idx = match after_name_span.find(verdict_marker) {
            Some(idx) => idx + verdict_marker.len(),
            None => continue,
        };
        let after_verdict_marker = &after_name_span[verdict_idx..];
        let verdict_start = match after_verdict_marker.find('>') {
            Some(idx) => idx + 1,
            None => continue,
        };
        let verdict_content = &after_verdict_marker[verdict_start..];
        let verdict_end = match verdict_content.find("</span>") {
            Some(idx) => idx,
            None => continue,
        };
        let verdict = verdict_content[..verdict_end].trim().to_string();

        if !name.is_empty() && !verdict.is_empty() {
            audits.push(SecurityAudit { name, verdict });
        }
    }

    audits
}

fn fetch_skills_sh_page(source: &str, skill_id: &str) -> Option<String> {
    let url = format!("https://www.skills.sh/{source}/{skill_id}");
    ureq::get(&url).call().ok()?.into_string().ok()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketplaceSkillPreview {
    pub description: String,
    pub when_to_use: String,
    pub body: String,
    /// True means the GitHub source is gone and only the description was
    /// recovered from skills.sh; when_to_use and body are empty in that case.
    pub source_gone: bool,
    /// Best-effort security audit verdicts scraped from skills.sh; empty when unavailable.
    pub security_audits: Vec<SecurityAudit>,
}

fn find_path_in_tree(tree_body: &str, skill_id: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_str(tree_body).ok()?;
    let suffix = format!("/{skill_id}/SKILL.md");
    let root = format!("{skill_id}/SKILL.md");
    doc.get("tree")?.as_array()?.iter().find_map(|e| {
        let p = e.get("path").and_then(|p| p.as_str())?;
        (p == root || p.ends_with(&suffix)).then(|| p.to_string())
    })
}

fn extract_skills_sh_description(html: &str) -> Option<String> {
    let tag = r#"<meta name="description" content=""#;
    let start = html.find(tag)? + tag.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    let content = rest[..end].trim();
    if content.is_empty() {
        return None;
    }
    let decoded = content
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    Some(decoded)
}

fn is_repo_gone(err: &ureq::Error) -> bool {
    matches!(err, ureq::Error::Status(404, _))
}

/// A 404 from GitHub here means the repo itself (or the file in it) is
/// gone — skills.sh's index lagging behind a repo that was since deleted,
/// renamed or made private — which reads very differently from "I can't
/// reach GitHub at all", so it gets its own message instead of a generic one.
fn github_error(lang: Lang, source: &str, err: ureq::Error) -> String {
    if is_repo_gone(&err) {
        i18n::skill_repo_gone(lang, source)
    } else {
        i18n::github_request_failed(lang, &err.to_string())
    }
}

/// Fetches just the one `SKILL.md` the user clicked to preview — a GitHub
/// tree lookup (to locate it, since a search result only has the skill's
/// folder name, not its path inside the repo) plus one raw-content fetch.
/// Deliberately not a full `git clone` like install does: previewing is a
/// read of a few KB of markdown, cloning the whole repo for that would be
/// wasteful for something the user might click through several of per search.
fn preview_marketplace_skill_impl(lang: Lang, source: &str, skill_id: &str) -> Result<MarketplaceSkillPreview, String> {
    let fallback = |e: ureq::Error| -> Result<MarketplaceSkillPreview, String> {
        if is_repo_gone(&e) {
            if let Some(page) = fetch_skills_sh_page(source, skill_id) {
                if let Some(description) = extract_skills_sh_description(&page) {
                    let security_audits = extract_security_audits(&page);
                    return Ok(MarketplaceSkillPreview {
                        description,
                        when_to_use: String::new(),
                        body: String::new(),
                        source_gone: true,
                        security_audits,
                    });
                }
            }
        }
        Err(github_error(lang, source, e))
    };

    let tree_response = match ureq::get(&format!("https://api.github.com/repos/{source}/git/trees/HEAD?recursive=1"))
        .set("User-Agent", "clman")
        .call()
    {
        Ok(res) => res,
        Err(e) => return fallback(e),
    };
    let tree_body = tree_response
        .into_string()
        .map_err(|e| i18n::github_request_failed(lang, &e.to_string()))?;
    let path = find_path_in_tree(&tree_body, skill_id).ok_or_else(|| i18n::skill_not_found_in_repo(lang, skill_id))?;

    let raw_response = match ureq::get(&format!("https://raw.githubusercontent.com/{source}/HEAD/{path}")).call() {
        Ok(res) => res,
        Err(e) => return fallback(e),
    };
    let raw = raw_response
        .into_string()
        .map_err(|e| i18n::github_request_failed(lang, &e.to_string()))?;

    let (yaml, body) = crate::fsutil::split_frontmatter(&raw);
    let fm: crate::models::SkillFrontmatter = yaml.as_deref().and_then(|y| serde_yaml::from_str(y).ok()).unwrap_or_default();
    let security_audits = fetch_skills_sh_page(source, skill_id)
        .map(|page| extract_security_audits(&page))
        .unwrap_or_default();

    Ok(MarketplaceSkillPreview {
        description: fm.description.unwrap_or_default(),
        when_to_use: fm.when_to_use.unwrap_or_default(),
        body,
        source_gone: false,
        security_audits,
    })
}

#[tauri::command]
pub async fn preview_marketplace_skill(
    state: State<'_, AppState>,
    source: String,
    skill_id: String,
) -> Result<MarketplaceSkillPreview, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || preview_marketplace_skill_impl(lang, &source, &skill_id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_marketplace_skill(
    state: State<'_, AppState>,
    source: String,
    skill_id: String,
    scope: Scope,
) -> Result<String, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || install_marketplace_skill_impl(&state, source, skill_id, scope))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search_response_reads_the_real_skills_sh_shape() {
        let body = r#"{"query":"test","searchType":"fuzzy","searchVersion":"algolia","skills":[
            {"id":"anthropics/skills/webapp-testing","source":"anthropics/skills","skillId":"webapp-testing","name":"webapp-testing","installs":177038}
        ]}"#;
        let skills = parse_search_response(body);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source, "anthropics/skills");
        assert_eq!(skills[0].skill_id, "webapp-testing");
        assert_eq!(skills[0].installs, 177038);
    }

    #[test]
    fn parse_search_response_handles_an_empty_or_malformed_body() {
        assert!(parse_search_response("not json").is_empty());
        assert!(parse_search_response(r#"{"skills":[]}"#).is_empty());
    }

    #[test]
    fn find_path_in_tree_matches_a_nested_skill_by_folder_name() {
        let tree = r#"{"tree":[
            {"path":"README.md","type":"blob"},
            {"path":"skills/webapp-testing/SKILL.md","type":"blob"},
            {"path":"skills/webapp-testing/reference.md","type":"blob"}
        ]}"#;
        assert_eq!(
            find_path_in_tree(tree, "webapp-testing"),
            Some("skills/webapp-testing/SKILL.md".to_string())
        );
        assert_eq!(find_path_in_tree(tree, "does-not-exist"), None);
    }

    #[test]
    fn find_path_in_tree_matches_a_root_level_skill() {
        let tree = r#"{"tree":[{"path":"my-skill/SKILL.md","type":"blob"}]}"#;
        assert_eq!(find_path_in_tree(tree, "my-skill"), Some("my-skill/SKILL.md".to_string()));
    }

    #[test]
    fn github_error_tells_a_gone_repo_apart_from_a_real_connectivity_failure() {
        let response = ureq::Response::new(404, "Not Found", "{}").unwrap();
        let msg = github_error(Lang::En, "marvinrichter/clarc", ureq::Error::Status(404, response));
        assert!(msg.contains("marvinrichter/clarc"), "should name the repo: {msg}");
        assert!(
            !msg.to_lowercase().contains("couldn't reach") && !msg.to_lowercase().contains("couldn't connect"),
            "a 404 (gone repo) isn't a connectivity failure, the message shouldn't read like one: {msg}"
        );
    }

    #[test]
    fn search_marketplace_skills_impl_short_circuits_a_too_short_query_without_a_request() {
        // Mirrors skills.sh's own "Query must be at least 2 characters" rule
        // so we don't burn a network round trip on every single keystroke.
        assert_eq!(search_marketplace_skills_impl(Lang::En, "a").unwrap(), Vec::new());
        assert_eq!(search_marketplace_skills_impl(Lang::En, " ").unwrap(), Vec::new());
    }

    #[test]
    fn extract_skills_sh_description_extracts_and_decodes() {
        let html = "<html><head><meta name=\"description\" content=\"Hexagonal architecture (ports &amp; adapters) for Java Spring Boot...\"></head></html>";
        assert_eq!(
            extract_skills_sh_description(html),
            Some("Hexagonal architecture (ports & adapters) for Java Spring Boot...".to_string())
        );
    }

    #[test]
    fn extract_skills_sh_description_decodes_entities_correctly() {
        let html = r#"<meta name="description" content="&quot;quotes&#39; &amp; &apos;apostrophes&apos; &lt;brackets&gt; &amp;lt;">"#;
        assert_eq!(
            extract_skills_sh_description(html),
            Some("\"quotes' & 'apostrophes' <brackets> &lt;".to_string())
        );
    }

    #[test]
    fn extract_skills_sh_description_returns_none_when_tag_absent_or_empty() {
        assert_eq!(extract_skills_sh_description("<html><head></head></html>"), None);
        assert_eq!(extract_skills_sh_description(r#"<meta name="description" content="">"#), None);
        assert_eq!(extract_skills_sh_description(r#"<meta name="description" content="   ">"#), None);
    }

    #[test]
    fn is_repo_gone_detects_404_only() {
        let resp_404 = ureq::Response::new(404, "Not Found", "{}").unwrap();
        let err_404 = ureq::Error::Status(404, resp_404);
        assert!(is_repo_gone(&err_404));

        let resp_500 = ureq::Response::new(500, "Internal Server Error", "{}").unwrap();
        let err_500 = ureq::Error::Status(500, resp_500);
        assert!(!is_repo_gone(&err_500));
    }

    #[test]
    fn extract_security_audits_parses_pass_and_warn_verdicts() {
        let html = r#"
<div class="text-sm font-mono uppercase text-white mb-3">Security Audits</div>
<div class="divide-y divide-border">
  <a class="block py-2 hover:bg-muted/40 -mx-3 px-3 rounded transition-colors border-0" href="/anthropics/skills/pdf/security/agent-trust-hub">
    <div class="flex items-center justify-between mb-1">
      <span class="text-sm font-medium text-foreground truncate">Gen Agent Trust Hub</span>
      <span class="text-xs font-mono uppercase px-2 py-1 rounded bg-green-500/10 text-green-500">Pass</span>
    </div>
  </a>
  <a class="block py-2 hover:bg-muted/40 -mx-3 px-3 rounded transition-colors border-0" href="/anthropics/skills/pdf/security/snyk">
    <div class="flex items-center justify-between mb-1">
      <span class="text-sm font-medium text-foreground truncate">Snyk</span>
      <span class="text-xs font-mono uppercase px-2 py-1 rounded bg-amber-500/10 text-amber-500">Warn</span>
    </div>
  </a>
</div>
"#;
        let audits = extract_security_audits(html);
        assert_eq!(
            audits,
            vec![
                SecurityAudit {
                    name: "Gen Agent Trust Hub".to_string(),
                    verdict: "Pass".to_string(),
                },
                SecurityAudit {
                    name: "Snyk".to_string(),
                    verdict: "Warn".to_string(),
                },
            ]
        );
    }

    #[test]
    fn extract_security_audits_returns_empty_when_section_absent() {
        let html = "<div>No audits here</div>";
        assert_eq!(extract_security_audits(html), Vec::new());
    }

    #[test]
    fn extract_security_audits_does_not_panic_on_truncated_block() {
        let test_cases = vec![
            "Security Audits",
            "Security Audits <div",
            "Security Audits <a href=\"foo\">",
            "Security Audits <a class=\"foo\"><span class=\"text-sm font-medium\">Truncated",
            "Security Audits <a class=\"foo\"><span class=\"text-sm font-medium\">Name</span><span class=\"text-xs font-mono uppercase\"></a>",
            "Security Audits <a class=\"foo\"><span class=\"text-sm font-medium\">   </span><span class=\"text-xs font-mono uppercase\">Pass</span></a>",
            "Security Audits <a class=\"foo\"><span class=\"text-sm font-medium\">Name</span><span class=\"text-xs font-mono uppercase\">   </span></a>",
        ];
        for html in test_cases {
            let res = extract_security_audits(html);
            for entry in res {
                assert!(!entry.name.trim().is_empty());
                assert!(!entry.verdict.trim().is_empty());
            }
        }
    }
}
