use sqlrustgo_storage::Record;
use sqlrustgo_types::Value;

#[derive(Debug, Clone)]
pub struct TriggerContext<'a> {
    new: Option<&'a Record>,
    old: Option<&'a Record>,
    new_col_names: Option<Vec<String>>,
    old_col_names: Option<Vec<String>>,
}

impl<'a> TriggerContext<'a> {
    pub fn new(new: Option<&'a Record>, old: Option<&'a Record>) -> Self {
        Self {
            new,
            old,
            new_col_names: None,
            old_col_names: None,
        }
    }

    pub fn with_new_col_names(mut self, names: Vec<String>) -> Self {
        self.new_col_names = Some(names);
        self
    }

    pub fn with_old_col_names(mut self, names: Vec<String>) -> Self {
        self.old_col_names = Some(names);
        self
    }

    pub fn new_row(&self) -> Option<&Record> {
        self.new
    }

    pub fn old_row(&self) -> Option<&Record> {
        self.old
    }

    pub fn new_col_names(&self) -> Option<&[String]> {
        self.new_col_names.as_deref()
    }

    pub fn old_col_names(&self) -> Option<&[String]> {
        self.old_col_names.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<'a> {
    trigger_ctx: &'a TriggerContext<'a>,
    target_row: Option<&'a Record>,
    target_col_names: Option<Vec<String>>,
}

impl<'a> EvalContext<'a> {
    pub fn new(trigger_ctx: &'a TriggerContext<'a>, target_row: Option<&'a Record>) -> Self {
        Self {
            trigger_ctx,
            target_row,
            target_col_names: None,
        }
    }

    pub fn with_target_col_names(mut self, names: Vec<String>) -> Self {
        self.target_col_names = Some(names);
        self
    }

    pub fn trigger(&self) -> &TriggerContext<'a> {
        self.trigger_ctx
    }

    pub fn target_row(&self) -> Option<&Record> {
        self.target_row
    }

    pub fn target_col_names(&self) -> Option<&[String]> {
        self.target_col_names.as_deref()
    }
}
