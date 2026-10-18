use crate::syntax::Id;

#[derive(Debug, PartialEq)]
pub(crate) enum Subject {
    Unit(Id, UnitState),
    Tool(Id),
}

#[derive(Debug, PartialEq)]
pub(crate) enum UnitState {
    Start,
    Finish,
    Fail,
    Watch,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    Subject(Subject),
    NOT(Box<Expression>),
    AND((Box<Expression>, Option<Box<Expression>>)),
    OR((Box<Expression>, Option<Box<Expression>>)),
}

fn push_split<'a>(rest: &mut Vec<&'a str>, word: &'a str) {
    let mut start = 0;
    for (i, c) in word.char_indices() {
        if c == '(' || c == ')' {
            if start < i {
                rest.push(&word[start..i]);
            }
            let end = i + c.len_utf8();
            rest.push(&word[i..end]);
            start = end;
        }
    }
    if start < word.len() {
        rest.push(&word[start..]);
    }
}

impl Expression {
    pub(crate) fn parser(when: &str) -> Result<Expression, String> {
        let mut rest = Vec::new();
        for word in when.trim().split_whitespace() {
            push_split(&mut rest, word);
        }
        if rest.is_empty() {
            return Err(format!("expression is empty"));
        }

        let mut expression: Option<Expression> = None;
        let parsed = none_parse(&mut expression, &mut rest)?;
        if !rest.is_empty() {
            return Err(format!("expected an operator, got '{}'", rest[0]));
        }
        Ok(parsed)
    }
}

fn none_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    if rest[0] == "AND" || rest[0] == "OR" {
        return Err(format!("{} missing arguments", rest[0]));
    }

    let subject = subject_parse(expression, rest)?;
    *expression = Some(subject);
    operator_parse(expression, rest)
}

fn not_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    rest.remove(0);

    if rest.is_empty() {
        return Err(format!("NOT missing arguments"));
    }

    Ok(Expression::NOT(Box::new(subject_parse(expression, rest)?)))
}

fn subject_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    if rest.is_empty() {
        return Err(format!("expected a Subject or NOT, got ''"));
    }

    if rest[0] == "NOT" {
        return not_parse(expression, rest);
    }

    if rest[0] == "(" {
        return group_parse(expression, rest);
    }

    let word = rest.remove(0);
    Ok(Expression::Subject(Subject::parser(word)?))
}

fn group_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    rest.remove(0);

    if rest.is_empty() || rest[0] == ")" {
        return Err(format!(
            "expected a Subject or NOT, got '{}'",
            rest.first().copied().unwrap_or("")
        ));
    }

    let inner = none_parse(expression, rest)?;
    if rest.first().copied() != Some(")") {
        return Err(format!(
            "expected ')', got '{}'",
            rest.first().copied().unwrap_or("")
        ));
    }
    rest.remove(0);
    Ok(inner)
}

fn operator_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    if rest.is_empty() || rest[0] == ")" {
        return expression
            .take()
            .ok_or(format!("expected a subject, got ''"));
    }

    match rest[0] {
        "AND" => and_parse(expression, rest),
        "OR" => or_parse(expression, rest),
        _ => Err(format!("expected an operator, got '{}'", rest[0])),
    }
}

fn or_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    let left = expression
        .take()
        .ok_or(format!("operator {} missing arguments", rest[0]))?;

    rest.remove(0);

    let right = subject_parse(expression, rest)?;

    *expression = Some(Expression::OR((Box::new(left), Some(Box::new(right)))));
    operator_parse(expression, rest)
}

fn and_parse(
    expression: &mut Option<Expression>,
    rest: &mut Vec<&str>,
) -> Result<Expression, String> {
    let left = expression
        .take()
        .ok_or(format!("operator {} missing arguments", rest[0]))?;

    rest.remove(0);

    let right = subject_parse(expression, rest)?;
    *expression = Some(right);

    while rest.first().copied() == Some("OR") {
        let or_left = expression
            .take()
            .ok_or(format!("operator {} missing arguments", rest[0]))?;

        rest.remove(0);

        let or_right = subject_parse(expression, rest)?;
        *expression = Some(Expression::OR((
            Box::new(or_left),
            Some(Box::new(or_right)),
        )));
    }

    let right = expression
        .take()
        .ok_or(format!("operator AND missing arguments"))?;

    *expression = Some(Expression::AND((Box::new(left), Some(Box::new(right)))));
    operator_parse(expression, rest)
}

impl Subject {
    fn parser(subject: &str) -> Result<Subject, String> {
        if subject.starts_with(":") || !subject.contains(":") {
            return Ok(Subject::Tool(Id::new_tool(subject)?));
        }

        if subject.contains(":") {
            if let Some(split) = subject.split_once(":") {
                let (unit, state) = split;
                return Ok(Subject::Unit(
                    Id::new_unit(unit)?,
                    UnitState::parser(state)?,
                ));
            }
        }

        Err(format!("expected tool or unit subject, got {subject}"))
    }
}

impl UnitState {
    pub(super) fn parser(state: &str) -> Result<Self, String> {
        match state {
            "start" => Ok(Self::Start),
            "finish" => Ok(Self::Finish),
            "fail" => Ok(Self::Fail),
            "watch" => Ok(Self::Watch),
            _ => Err(format!(
                "State expected 'start' 'fìnish' 'fail' 'watch', got '{state}'"
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rejected() {
        for i in ["", "OR", "AND", "NOT"] {
            assert!(Expression::parser(i).is_err());
        }
    }

    #[test]
    fn subject() {
        for i in [
            "main:watch",
            "build:start",
            "build:fail",
            ":bash",
            "curl",
            "safe_command",
        ] {
            assert_eq!(
                Expression::parser(i).unwrap(),
                Expression::Subject(Subject::parser(i).unwrap()),
            );
        }
    }

    #[test]
    fn not() {
        let subjects = vec!["build:start", "my_command", ":bash", "main:watch"];
        for i in subjects {
            assert_eq!(
                Expression::parser(&format!("NOT {i}")).unwrap(),
                Expression::NOT(Box::new(Expression::Subject(Subject::parser(i).unwrap())))
            );
        }
    }

    #[test]
    fn nested() {
        assert_eq!(
            Expression::parser("(((NOT NOT dev:start)))").unwrap(),
            Expression::NOT(Box::new(Expression::NOT(Box::new(Expression::Subject(
                Subject::parser("dev:start").unwrap(),
            )))))
        );
    }

    #[test]
    fn chain() {
        fn sub(s: &str) -> Expression {
            Expression::Subject(Subject::parser(s).unwrap())
        }
        fn not(e: Expression) -> Expression {
            Expression::NOT(Box::new(e))
        }
        fn and(l: Expression, r: Expression) -> Expression {
            Expression::AND((Box::new(l), Some(Box::new(r))))
        }
        fn or(l: Expression, r: Expression) -> Expression {
            Expression::OR((Box::new(l), Some(Box::new(r))))
        }

        let parsed = Expression::parser(
            "NOT ((a:start OR b:finish) AND NOT c:fail) OR (d:watch AND curl OR :bash) AND NOT NOT (e:start OR :bash AND f:finish) OR (NOT a:start AND (b:finish OR NOT (c:fail AND d:watch))) AND ((NOT curl OR a:start) AND NOT (b:finish OR c:fail))",
        )
        .unwrap();

        let expected = and(
            and(
                or(
                    not(and(or(sub("a:start"), sub("b:finish")), not(sub("c:fail")))),
                    and(sub("d:watch"), or(sub("curl"), sub(":bash"))),
                ),
                or(
                    not(not(and(or(sub("e:start"), sub(":bash")), sub("f:finish")))),
                    and(
                        not(sub("a:start")),
                        or(sub("b:finish"), not(and(sub("c:fail"), sub("d:watch")))),
                    ),
                ),
            ),
            and(
                or(not(sub("curl")), sub("a:start")),
                not(or(sub("b:finish"), sub("c:fail"))),
            ),
        );

        assert_eq!(parsed, expected);
    }
}
