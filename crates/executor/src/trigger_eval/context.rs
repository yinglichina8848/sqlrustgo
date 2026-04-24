use sqlrustgo_storage::Record;
use sqlrustgo_types::Value;

#[derive(Debug, Clone)]
pub struct TriggerContext<'a> {
    new: Option<&'a Record>,
    old: Option<&'a Record>,
}

impl<'a> TriggerContext<'a> {
    pub fn new(new: Option<&'a Record>, old: Option<&'a Record>) -> Self {
        Self { new, old }
    }

    pub fn new_row(&self) -> Option<&Record> {
        self.new
    }

    pub fn old_row(&self) -> Option<&Record> {
        self.old
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<'a> {
    trigger_ctx: &'a TriggerContext<'a>,
    target_row: Option<&'a Record>,
}

impl<'a> EvalContext<'a> {
    pub fn new(
        trigger_ctx: &'a TriggerContext<'a>,
        target_row: Option<&'a Record>,
    ) -> Self {
        Self { trigger_ctx, target_row }
    }

    pub fn trigger(&self) -> &TriggerContext<'a> {
        self.trigger_ctx
    }

    pub fn target_row(&self) -> Option<&Record> {
        self.target_row
    }
}
