use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct SelectOptionAction;

#[derive(Deserialize)]
struct Params {
    #[serde(alias = "value", alias = "option", alias = "label")]
    text: String,
    #[serde(default, alias = "element_index", alias = "H_index")]
    index: Option<i64>,
}

const SELECT_JS: &str = r#"(function(){
  var want = __WANT__;
  var wn = String(want).trim().toLowerCase();
  var node = __NODE__;
  function txt(el){ return String(el.innerText || el.textContent || "").trim(); }
  if (node && node.tagName === "SELECT") {
    var opts = Array.prototype.slice.call(node.options);
    var opt = opts.find(function(o){ return txt(o).toLowerCase() === wn; })
           || opts.find(function(o){ return String(o.value || "").trim().toLowerCase() === wn; })
           || opts.find(function(o){ return txt(o).toLowerCase().indexOf(wn) !== -1; });
    if (!opt) { return { ok: false, kind: "select", options: opts.map(function(o){ return txt(o); }) }; }
    node.value = opt.value;
    node.dispatchEvent(new Event("input", { bubbles: true }));
    node.dispatchEvent(new Event("change", { bubbles: true }));
    return { ok: true, kind: "select", selected: txt(opt) };
  }
  var scope = node || document;
  var q = "[role=option],[role=menuitemradio],[role=menuitem],option,li,a,button,div,span";
  function pick(root){
    var els = Array.prototype.slice.call(root.querySelectorAll(q));
    return els.find(function(el){ return txt(el).toLowerCase() === wn; })
        || els.find(function(el){ var t = txt(el).toLowerCase(); return t.length < 120 && t.indexOf(wn) !== -1; });
  }
  var match = pick(scope) || (scope !== document ? pick(document) : null);
  if (!match) { return { ok: false, kind: "custom" }; }
  match.scrollIntoView({ block: "center" });
  match.click();
  return { ok: true, kind: "custom", selected: txt(match) };
})()"#;

fn js_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// Build the one-shot JS that selects an option by VISIBLE TEXT: a native
/// `<select>` (resolved by xpath) is set via value + input/change events;
/// otherwise the node subtree (or whole document) is searched for a matching
/// option, scrolled into view, and clicked. Text/xpath are JSON-encoded.
fn build_select_js(xpath: Option<&str>, text: &str) -> String {
    let node = match xpath {
        Some(xp) => format!(
            "document.evaluate({}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
            js_string(xp)
        ),
        None => "null".to_string(),
    };
    SELECT_JS
        .replace("__WANT__", &js_string(text))
        .replace("__NODE__", &node)
}

/// Turn the JS result envelope into an `ActionResult`: success names the chosen
/// option; failure is an error (listing the available native options if any).
fn interpret_result(value: &serde_json::Value, text: &str) -> Result<ActionResult, AppError> {
    if value.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        let selected = value
            .get("selected")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(text);
        return Ok(ActionResult::ok(format!("selected option {selected:?}")));
    }
    let kind = value
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("dropdown");
    let mut msg = format!("no option matching {text:?} in the {kind}");
    if let Some(options) = value.get("options").and_then(serde_json::Value::as_array) {
        let names: Vec<&str> = options
            .iter()
            .filter_map(serde_json::Value::as_str)
            .take(25)
            .collect();
        if !names.is_empty() {
            msg.push_str(&format!("; available: {}", names.join(", ")));
        }
    }
    Err(AppError::ActionFailed(msg))
}

#[async_trait]
impl ToolHandler for SelectOptionAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("select_option".into()),
            description: "Select a dropdown option by its VISIBLE TEXT (not by index). For a native <select>, pass its `index`; for a custom dropdown, open it first then pass the option `text` (with or without the open list's `index`).".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Visible text of the option to choose."},
                    "index": {"type": "integer", "description": "Clickable index of the <select> or dropdown container; omit to search the open list."}
                },
                "required": ["text"]
            }),
            domain_filter: Vec::new(),
            terminates_sequence: false,
            timeout: ActionTimeout::default(),
        }
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ActionResult, AppError> {
        let p: Params = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("select_option params: {e}")))?;
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        let xpath = match p.index {
            Some(idx) => {
                let element = ctx
                    .clickables
                    .iter()
                    .find(|c| i64::from(c.index) == idx)
                    .ok_or_else(|| {
                        AppError::ValidationError(format!(
                            "no clickable with index {} in current snapshot ({} available)",
                            idx,
                            ctx.clickables.len()
                        ))
                    })?;
                Some(element.xpath.clone()).filter(|x| !x.is_empty())
            }
            None => None,
        };
        let js = build_select_js(xpath.as_deref(), &p.text);
        let value = ctx.browser.evaluate(&target, &js).await?;
        interpret_result(&value, &p.text)
    }
}

#[cfg(test)]
mod tests {
    use super::{build_select_js, interpret_result};
    use serde_json::json;

    #[test]
    fn js_resolves_xpath_for_select_but_searches_document_without_one() {
        let with = build_select_js(Some("/html/body/select[1]"), "BCA Bank");
        assert!(with.contains("document.evaluate(") && with.contains("/html/body/select[1]"));
        assert!(with.contains("BCA Bank") && with.contains("tagName === \"SELECT\""));
        let without = build_select_js(None, "Savings");
        assert!(!without.contains("document.evaluate(") && without.contains("var node = null;"));
    }

    #[test]
    fn js_json_encodes_quotes_in_the_wanted_text() {
        assert!(build_select_js(None, "the \"big\" one").contains("the \\\"big\\\" one"));
    }

    #[test]
    fn interpret_names_selection_on_ok_and_lists_options_on_failure() {
        let ok = interpret_result(&json!({"ok": true, "selected": "BCA"}), "BCA").expect("ok");
        assert!(
            ok.extracted_content
                .as_deref()
                .unwrap_or_default()
                .contains("BCA")
        );
        let err = interpret_result(
            &json!({"ok": false, "kind": "select", "options": ["BNI", "BRI"]}),
            "Nope",
        )
        .expect_err("not-ok is an error");
        let msg = err.to_string();
        assert!(msg.contains("Nope") && msg.contains("BNI") && msg.contains("BRI"));
        assert!(interpret_result(&json!({}), "X").is_err());
    }
}
