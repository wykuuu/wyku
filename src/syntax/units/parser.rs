use super::when::Expression;
use crate::syntax::tools::parser::parse_tool;
use crate::syntax::{Id, Parsable, Unit};

use serde::Deserialize;
use toml::{Table, Value, value::Array};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitTypes {
    task: Option<String>,
    exec: Option<Array>,
    when: Option<String>,
    tools: Option<Vec<String>>,
}

impl Parsable<Vec<Unit>> for Unit {
    fn parser(toml: Table) -> Result<Vec<Unit>, String> {
        toml.into_iter()
            .map(|(key, value)| match value {
                Value::Table(body) => parse_unit(key, body).map_err(|e| e.to_string()),
                _ => Err(format!("unit `{key}` must be a table")),
            })
            .collect()
    }
}

fn parse_unit(id: String, table: Table) -> Result<Unit, String> {
    if id == "tools" {
        return Err("unit id cannot be `tools`".into());
    }

    let id = Id::new(&id)?;
    let raw = UnitTypes::deserialize(Value::Table(table)).map_err(|err| err.to_string())?;

    let exec = match raw.exec {
        Some(array) if !array.is_empty() => Some(parse_tool(array)?),
        _ => None,
    };

    let when = match raw.when.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(text) => Some(Expression::parser(text)?),
    };

    let tools = raw
        .tools
        .map(|names| names.iter().map(|name| Id::new_tool(name)).collect())
        .transpose()?;

    Ok(Unit {
        id,
        task: raw.task,
        exec,
        when,
        tools,
        state: None,
        ok: None,
    })
}
