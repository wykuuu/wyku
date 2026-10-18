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

    let id = Id::new_unit(&id)?;
    let raw = UnitTypes::deserialize(Value::Table(table)).map_err(|e| e.to_string())?;

    let exec = match raw.exec {
        Some(array) if !array.is_empty() => Some(parse_tool(None, array)?),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parser() {
        let parsed = Unit::parser(
            toml::from_str(
                r#"
                [main]
                task = "@(:user)"
                exec = [":bash", "echo hi"]
                when = "main:finish"
                tools = [":bash", "read"]
                "#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(parsed.len(), 1);
        let main = &parsed[0];
        assert_eq!(main.id, Id::new_unit("main").unwrap());
        assert_eq!(main.task.as_deref(), Some("@(:user)"));
        assert_eq!(
            main.exec,
            Some(
                parse_tool(
                    None,
                    vec![
                        Value::String(":bash".into()),
                        Value::String("echo hi".into()),
                    ]
                )
                .unwrap()
            )
        );
        assert_eq!(main.when, Some(Expression::parser("main:finish").unwrap()));
        assert_eq!(
            main.tools,
            Some(vec![
                Id::new_tool(":bash").unwrap(),
                Id::new_tool("read").unwrap(),
            ])
        );
        assert!(main.state.is_none());
        assert!(main.ok.is_none());
    }
}
