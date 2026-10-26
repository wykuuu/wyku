#[derive(Debug, PartialEq)]
pub(crate) struct Id(pub(super) String);

fn unit_algoritm(unit: &str) -> bool {
    unit != "tools" && unit.bytes().all(|c| matches!(c, b'a'..=b'z' | b'_' | b'-'))
}

fn tool_algoritm(tool: &str) -> bool {
    let w = |p: &str| !p.is_empty() && p.bytes().all(|c| matches!(c, b'a'..=b'z' | b'_' | b'-'));
    tool.strip_prefix(':').map_or_else(
        || {
            tool.split_once("::")
                .map_or_else(|| w(tool), |(a, b)| w(a) && w(b))
        },
        w,
    )
}

impl Id {
    pub(crate) fn new_unit(id: &str) -> Result<Id, String> {
        Id::new(id, unit_algoritm)
    }

    pub(crate) fn new_tool(id: &str) -> Result<Id, String> {
        Id::new(id, tool_algoritm)
    }

    fn new(id: &str, algoritm: fn(&str) -> bool) -> Result<Id, String> {
        if id.is_empty() {
            return Err(format!("void Id"));
        }
        if !algoritm(id) {
            return Err(format!("invalid id '{id}'"));
        }

        Ok(Id(id.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_units_ids() {
        let ids = vec!["Hello World", "Tooling", "Ñ", "Agent", "tools"];

        for i in ids {
            let id = Id::new_unit(i);
            assert!(id.is_err())
        }
    }

    #[test]
    fn valid_units_ids() {
        let ids = vec!["main", "check", "worker", "agent"];

        for i in ids {
            let id = Id::new_unit(i);
            assert!(id.is_ok())
        }
    }
}
