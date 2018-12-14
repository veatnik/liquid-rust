use std::fmt;
use std::io::Write;

use itertools;

use liquid_error::{Result, ResultLiquidChainExt, ResultLiquidExt};
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_interpreter::Renderable;
use liquid_value::Value;

use super::BoxedValueFilter;
use super::FilterValue;

/// A `Value` filter.
#[derive(Clone, Debug)]
pub struct FilterCall {
    name: String,
    filter: BoxedValueFilter,
    arguments: Vec<Expression>,
}

impl FilterCall {
    /// Create filter expression.
    pub fn new(name: &str, filter: BoxedValueFilter, arguments: Vec<Expression>) -> FilterCall {
        FilterCall {
            name: name.to_owned(),
            filter,
            arguments,
        }
    }

    pub fn evaluate(&self, context: &Context, entry: &Value) -> Result<Value> {
        let arguments: Result<Vec<Value>> = self
            .arguments
            .iter()
            .map(|a| Ok(a.evaluate(context)?.to_owned()))
            .collect();
        let arguments = arguments?;
        self.filter
            .filter(entry, &*arguments)
            .chain("Filter error")
            .context_key("filter")
            .value_with(|| format!("{}", self).into())
            .context_key("input")
            .value_with(|| format!("{}", &entry).into())
            .context_key("args")
            .value_with(|| itertools::join(&arguments, ", ").into())
    }
}

impl fmt::Display for FilterCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            itertools::join(&self.arguments, ", ")
        )
    }
}

/// A `Value` expression.
#[derive(Clone, Debug)]
pub struct FilterChain {
    entry: Expression,
    filters: Vec<FilterCall>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(entry: Expression, filters: Vec<FilterCall>) -> Self {
        Self { entry, filters }
    }

    /// Process `Value` expression within `context`'s stack.
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        // take either the provided value or the value from the provided variable
        let mut entry = self.entry.evaluate(context)?.to_owned();

        // apply all specified filters
        for filter in &self.filters {
            entry = filter.evaluate(context, &entry)?;
        }

        Ok(entry)
    }
}

impl fmt::Display for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.entry,
            itertools::join(&self.filters, " | ")
        )
    }
}

impl Renderable for FilterChain {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let entry = self.evaluate(context)?;
        write!(writer, "{}", entry).chain("Failed to render")?;
        Ok(())
    }
}