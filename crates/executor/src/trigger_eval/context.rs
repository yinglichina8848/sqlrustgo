use sqlrustgo_storage::Record;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::Record;
    use sqlrustgo_types::Value;

    fn make_record(values: &[i64]) -> Record {
        values.iter().map(|&v| Value::Integer(v)).collect()
    }

    #[test]
    fn test_trigger_context_new_row_only() {
        let new_record = make_record(&[1, 100, 5000]);
        let ctx = TriggerContext::new(Some(&new_record), None);
        assert_eq!(ctx.new_row(), Some(&new_record));
        assert_eq!(ctx.old_row(), None);
    }

    #[test]
    fn test_trigger_context_old_row_only() {
        let old_record = make_record(&[1, 50, 3000]);
        let ctx = TriggerContext::new(None, Some(&old_record));
        assert_eq!(ctx.new_row(), None);
        assert_eq!(ctx.old_row(), Some(&old_record));
    }

    #[test]
    fn test_trigger_context_both_rows() {
        let old_record = make_record(&[1, 50, 3000]);
        let new_record = make_record(&[1, 100, 5000]);
        let ctx = TriggerContext::new(Some(&new_record), Some(&old_record));
        assert_eq!(ctx.new_row(), Some(&new_record));
        assert_eq!(ctx.old_row(), Some(&old_record));
    }

    #[test]
    fn test_trigger_context_col_names() {
        let new_record = make_record(&[1, 100]);
        let old_record = make_record(&[1, 50]);
        let ctx = TriggerContext::new(Some(&new_record), Some(&old_record))
            .with_new_col_names(vec!["id".into(), "amount".into()])
            .with_old_col_names(vec!["id".into(), "amount".into()]);

        assert_eq!(
            ctx.new_col_names(),
            Some(&["id".into(), "amount".into()][..])
        );
        assert_eq!(
            ctx.old_col_names(),
            Some(&["id".into(), "amount".into()][..])
        );
    }

    #[test]
    fn test_eval_context_basic() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        assert!(eval.target_row().is_none());
        assert!(eval.target_col_names().is_none());
        assert_eq!(eval.trigger().new_row(), None);
    }

    #[test]
    fn test_eval_context_with_target() {
        let trigger_ctx = TriggerContext::new(None, None);
        let target = make_record(&[10, 20, 30]);
        let eval = EvalContext::new(&trigger_ctx, Some(&target)).with_target_col_names(vec![
            "a".into(),
            "b".into(),
            "c".into(),
        ]);

        assert_eq!(eval.target_row(), Some(&target));
        assert_eq!(
            eval.target_col_names(),
            Some(&["a".into(), "b".into(), "c".into()][..])
        );
    }
}
