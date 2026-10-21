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
        let mut units = Vec::new();

        for (key, value) in toml {
            let Value::Table(body) = value else {
                return Err(format!("'{key}' must be in '[tools]' instead"));
            };

            if !body.values().any(Value::is_table) {
                units.push(parse_unit(vec![key], body)?);
                continue;
            }

            if body.values().any(|v| !v.is_table()) {
                return Err(format!("unit '{key}' cannot have tables"));
            }

            for (child, value) in body {
                let Value::Table(table) = value else {
                    unreachable!()
                };
                if table.values().any(Value::is_table) {
                    return Err(format!("unit '{key}.{child}' cannot have tables"));
                }

                units.push(parse_unit(vec![key.clone(), child], table)?);
            }
        }

        Ok(units)
    }
}

fn parse_unit(id: Vec<String>, table: Table) -> Result<Unit, String> {
    let id = id
        .into_iter()
        .map(|id| Id::new_unit(&id))
        .rev() // the Id is the first element
        .collect::<Result<Vec<_>, _>>()?;

    let raw = UnitTypes::deserialize(Value::Table(table)).map_err(|e| e.to_string())?;

    let task = raw.task.unwrap_or("@(:prompt)".into());

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
        task,
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
        assert_eq!(main.id, vec![Id::new_unit("main").unwrap()]);
        assert_eq!(main.task, "@(:user)");
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
