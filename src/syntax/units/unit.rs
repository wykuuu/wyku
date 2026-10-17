use crate::syntax::when::{Expression, UnitState};

use crate::syntax::Id;
use crate::syntax::Tool;

#[derive(Debug)]
pub(crate) struct Unit {
    pub(super) id: Id,
    pub(super) task: Option<String>,
    pub(super) exec: Option<Tool>,
    pub(super) when: Option<Expression>,
    pub(super) tools: Option<Vec<Id>>,

    pub(super) state: Option<UnitState>,
    pub(super) ok: Option<bool>,
}
