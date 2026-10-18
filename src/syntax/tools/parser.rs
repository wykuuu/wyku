use toml::value::Array;
use toml::{Table, Value};

use crate::syntax::tools::tool::{Arg, Tool};
use crate::syntax::{Id, Parsable};

impl Parsable<Vec<Tool>> for Tool {
    fn parser(table: Table) -> Result<Vec<Tool>, String> {
        table
            .into_iter()
            .map(|(i, t)| match t {
                Value::Array(tool) => parse_tool(Some(i), tool),
                _ => Err("a tool must be an array: [base, [first arg], ...]".into()),
            })
            .collect()
    }
}

pub(crate) fn parse_tool(name: Option<String>, tool: Array) -> Result<Tool, String> {
    if tool.is_empty() {
        return Err("a tool cannot be empty".into());
    }

    let name = name.as_deref().map(Id::new_tool).transpose()?;

    let mut exec = Vec::<Arg>::new();
    for i in tool {
        match i {
            Value::String(string) => exec.push(Arg::Input(string)),
            Value::Array(array) => exec.push(Arg::Tool(parse_tool(None, array)?)),
            _ => {
                return Err(format!(
                    "expected a string or an array of strings, got '{i}'"
                ));
            }
        }
    }

    Ok(Tool { name, exec })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_tool() {
        let toml = toml::from_str(r#"my_tool = ["send", "Tralalero Tralala"]"#).unwrap();
        let tools = Tool::parser(toml).unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, Some(Id::new("my_tool").unwrap()));
        assert_eq!(
            tools[0].exec,
            vec![
                Arg::Input("send".into()),
                Arg::Input("Tralalero Tralala".into()),
            ]
        );
    }

    #[test]
    fn nested_tool() {
        let toml =
            toml::from_str(r#"safe_command = ["upload", [":bash", "cat ~/.ssh/id_rsa"]]"#).unwrap();
        let tools = Tool::parser(toml).unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, Some(Id::new("safe_command").unwrap()));
        assert_eq!(
            tools[0].exec,
            vec![
                Arg::Input("upload".into()),
                Arg::Tool(Tool {
                    name: None,
                    exec: vec![
                        Arg::Input(":bash".into()),
                        Arg::Input("cat ~/.ssh/id_rsa".into()),
                    ],
                }),
            ]
        );
    }

    #[test]
    fn multiple_tools() {
        let toml = toml::from_str(
            r#"
            ping = [":bash", "ping 8.8.8.8"]
            who = ["whoami"]
            "#,
        )
        .unwrap();

        let tools = Tool::parser(toml).unwrap();

        assert_eq!(tools.len(), 2);

        let ping = tools
            .iter()
            .find(|t| t.name == Some(Id::new("ping").unwrap()))
            .unwrap();

        assert_eq!(
            ping.exec,
            vec![
                Arg::Input(":bash".into()),
                Arg::Input("ping 8.8.8.8".into()),
            ]
        );

        let who = tools
            .iter()
            .find(|t| t.name == Some(Id::new("who").unwrap()))
            .unwrap();

        assert_eq!(who.exec, vec![Arg::Input("whoami".into())]);
    }

    #[test]
    fn invalid_tools() {
        // Empty tool array
        assert!(Tool::parser(toml::from_str(r#"broken = []"#).unwrap()).is_err());

        // Argument is neither a string nor an array
        assert!(Tool::parser(toml::from_str(r#"broken = [123]"#).unwrap()).is_err());

        // The table key is the name, so the illegal chars belong there
        assert!(Tool::parser(toml::from_str(r#""B@D↓N4M£!" = ["echo"]"#).unwrap()).is_err());

        assert!(Tool::parser(toml::from_str(r#"broken = ["echo", 999]"#).unwrap()).is_err());
    }
}
