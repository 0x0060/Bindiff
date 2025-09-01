use serde::Serialize;
use anyhow::Result;
use crate as bindiff_report; // for docs
use bindiff_core::diff::{DiffResult, FunctionDelta, MatchKind};

const HTML_TEMPLATE: &str = r#"
<!doctype html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>bindiff report</title>
<style>
body { font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Helvetica, Arial, Apple Color Emoji, Segoe UI Emoji; margin: 2rem; }
.summary { display: flex; gap: 1rem; }
.card { border: 1px solid #ddd; border-radius: 12px; padding: 1rem; }
.badge { padding: .25rem .5rem; border-radius: 8px; font-weight: 700; }
.badge.green { background: #d1fae5; color: #065f46; }
.badge.yellow { background: #fef9c3; color: #854d0e; }
.badge.red { background: #fee2e2; color: #991b1b; }
.badge.blue { background: #dbeafe; color: #1e3a8a; }
pre { background: #0b1020; color: #d1d5db; padding: 1rem; border-radius: 8px; overflow-x: auto; }
h2 { margin-top: 2rem; }
.fn { margin-bottom: 1rem; }
.fn h3 { margin: 0; font-size: 1rem; }
</style>
</head>
<body>
<h1>bindiff report</h1>
<div class='summary'>
  <div class='card'><span class='badge green'>Unchanged</span> {{unchanged}}</div>
  <div class='card'><span class='badge yellow'>Modified</span> {{modified}}</div>
  <div class='card'><span class='badge blue'>Added</span> {{added}}</div>
  <div class='card'><span class='badge red'>Removed</span> {{removed}}</div>
</div>

<h2>Modified</h2>
{{#each modified_items}}
<div class='fn card'>
  <h3>{{name}}</h3>
  {{#if hamming}}<div>SimHash Hamming distance: {{hamming}}</div>{{/if}}
  {{#if diff}}
  <details open><summary>Unified diff</summary>
  <pre>{{diff}}</pre>
  </details>
  {{/if}}
</div>
{{/each}}

<h2>Added</h2>
<ul>
{{#each added_items}}
  <li class='card fn'>{{name}}</li>
{{/each}}
</ul>

<h2>Removed</h2>
<ul>
{{#each removed_items}}
  <li class='card fn'>{{name}}</li>
{{/each}}
</ul>

<h2>Unchanged</h2>
<ul>
{{#each unchanged_items}}
  <li class='card fn'>{{name}}</li>
{{/each}}
</ul>

</body>
</html>
"#;

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}

pub fn render_html(result: &DiffResult) -> Result<String> {
    let summary = format!(
        "{}|{}|{}|{}",
        result.unchanged.len(), result.modified.len(), result.added.len(), result.removed.len()
    );
    let mut html = HTML_TEMPLATE.to_string();
    html = html.replace("{{unchanged}}", &result.unchanged.len().to_string());
    html = html.replace("{{modified}}", &result.modified.len().to_string());
    html = html.replace("{{added}}", &result.added.len().to_string());
    html = html.replace("{{removed}}", &result.removed.len().to_string());

    let mut modified_block = String::new();
    for m in &result.modified {
        let name = format!("{} → {}", m.name_a.clone().unwrap_or("?".into()), m.name_b.clone().unwrap_or("?".into()));
        let ham = match m.kind {
            MatchKind::Fuzzy { hamming } => Some(hamming),
            _ => None,
        };
        let diff = m.unified_diff.as_deref().map(escape_html);
        modified_block.push_str("<div class='fn card'>");
        modified_block.push_str(&format!("<h3>{}</h3>", escape_html(&name)));
        if let Some(h) = ham {
            modified_block.push_str(&format!("<div>SimHash Hamming distance: {}</div>", h));
        }
        if let Some(d) = diff {
            modified_block.push_str("<details open><summary>Unified diff</summary><pre>");
            modified_block.push_str(&d);
            modified_block.push_str("</pre></details>");
        }
        modified_block.push_str("</div>");
    }
    html = html.replacen(
        "{{#each modified_items}}\n<div class='fn card'>\n  <h3>{{name}}</h3>\n  {{#if hamming}}<div>SimHash Hamming distance: {{hamming}}</div>{{/if}}\n  {{#if diff}}\n  <details open><summary>Unified diff</summary>\n  <pre>{{diff}}</pre>\n  </details>\n  {{/if}}\n</div>\n{{/each}}",
        &modified_block,
        1
    );

    let list_block = |items: &Vec<String>| -> String {
        let mut s = String::new();
        for it in items {
            s.push_str("<li class='card fn'>");
            s.push_str(&escape_html(it));
            s.push_str("</li>");
        }
        s
    };

    let added_items: Vec<String> = result.added.iter().map(|d| d.name_b.clone().unwrap_or("?".into())).collect();
    let removed_items: Vec<String> = result.removed.iter().map(|d| d.name_a.clone().unwrap_or("?".into())).collect();
    let unchanged_items: Vec<String> = result.unchanged.iter().map(|d| d.name_a.clone().unwrap_or("?".into())).collect();

    html = html.replacen("{{#each added_items}}\n  <li class='card fn'>{{name}}</li>\n{{/each}}", &list_block(&added_items), 1);
    html = html.replacen("{{#each removed_items}}\n  <li class='card fn'>{{name}}</li>\n{{/each}}", &list_block(&removed_items), 1);
    html = html.replacen("{{#each unchanged_items}}\n  <li class='card fn'>{{name}}</li>\n{{/each}}", &list_block(&unchanged_items), 1);

    Ok(html)
}
