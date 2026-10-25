//! Graph processing of `[tool]` and `[toolset]` config with `ToolService`.

use std::io::Write;

use anyhow::Result;
use composable_runtime::composition::graph::Node;
use composable_runtime::{ComponentGraph, Service};
use composable_tools::ToolService;

/// The graph built from `toml`, written to a temp file named for `test`.
fn graph(test: &str, toml: &str) -> Result<ComponentGraph> {
    let mut file = tempfile::Builder::new()
        .prefix(&format!("composable-tools-{test}-"))
        .suffix(".toml")
        .tempfile()?;
    file.write_all(toml.as_bytes())?;
    let handler = ToolService.config_handler().expect("a config handler");
    ComponentGraph::builder()
        .from_path(file.path())
        .add_handler(handler)
        .build()
}

#[test]
fn configured_tools_and_toolsets_define_components() -> Result<()> {
    let graph = graph(
        "define",
        r#"
        [component.weather]
        uri = "weather.wasm"

        [tool.forecast]
        target = "weather"
        description = "The forecast"

        [tool.alerts]
        target = "weather"

        [toolset.outlook]
        tools = ["forecast", "alerts"]
        "#,
    )?;

    let mut names: Vec<&str> = graph
        .nodes()
        .filter_map(|node| match &node.weight {
            Node::Component(definition) => Some(definition.name.as_str()),
            _ => None,
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "_alerts-function",
            "_alerts-function-factory",
            "_forecast-function",
            "_forecast-function-factory",
            "_json-mapper",
            "_outlook-factory",
            "alerts",
            "forecast",
            "outlook",
            "weather",
        ]
    );
    Ok(())
}

#[test]
fn a_tool_rejects_a_property_it_does_not_claim() {
    let error = graph(
        "unclaimed",
        r#"
        [component.weather]
        uri = "weather.wasm"

        [tool.forecast]
        target = "weather"
        units = "metric"
        "#,
    )
    .expect_err("an unknown property")
    .to_string();
    assert!(error.contains("unknown property 'units'"), "{error}");
}
