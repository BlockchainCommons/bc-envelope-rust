use anyhow::{bail, Error, Result};

use dcbor::prelude::*;

use crate::{Envelope, EnvelopeEncodable, Function, Parameter};

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    function: Function,
    envelope: Envelope,
}

impl Expression {
    pub fn new(function: impl Into<Function>) -> Self {
        let function = function.into();
        Self {
            function: function.clone(),
            envelope: Envelope::new(function),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.envelope.format())
    }
}

pub trait ExpressionBehavior {
    //
    // Composition
    //

    /// Adds a parameter to the expression.
    fn with_parameter(self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self;

    /// Adds a parameter to the expression, if the value is not `None`.
    fn with_optional_parameter(self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self;

    //
    // Parsing
    //

    /// Returns the function of the expression.
    fn function(&self) -> &Function;

    /// Returns the envelope of the expression.
    fn expression_envelope(&self) -> &Envelope;

    /// Returns the argument for the given parameter.
    fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope>;

    /// Returns the arguments for the given possibly repeated parameter.
    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope>;

    /// Returns the argument for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is not exactly one matching `parameter`,
    /// or if the parameter value is not the correct type.
    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static;

    /// Returns the argument for the given parameter, or `None` if there is no matching parameter.
    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(
        &self,
        param: impl Into<Parameter>,
    ) -> Result<Option<T>>;

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = Error> + 'static;
}

impl ExpressionBehavior for Expression {
    fn with_parameter(
        mut self,
        parameter: impl Into<Parameter>,
        value: impl EnvelopeEncodable,
    ) -> Self {
        let assertion = Envelope::new_assertion(parameter.into(), value.into_envelope());
        self.envelope = self.envelope.add_assertion_envelope(assertion).unwrap();
        self
    }

    fn with_optional_parameter(
        self,
        parameter: impl Into<Parameter>,
        value: Option<impl EnvelopeEncodable>,
    ) -> Self {
        if let Some(value) = value {
            return self.with_parameter(parameter, value);
        }
        self
    }

    fn function(&self) -> &Function {
        &self.function
    }

    fn expression_envelope(&self) -> &Envelope {
        &self.envelope
    }

    fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope> {
        self.envelope.object_for_predicate(param.into())
    }

    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.envelope.objects_for_predicate(param.into())
    }

    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.envelope.extract_object_for_predicate(param.into())
    }

    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(
        &self,
        param: impl Into<Parameter>,
    ) -> Result<Option<T>> {
        self.envelope
            .extract_optional_object_for_predicate(param.into())
    }

    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.envelope.extract_objects_for_predicate(param.into())
    }
}

/// Expression -> Envelope
impl From<Expression> for Envelope {
    fn from(expression: Expression) -> Self {
        expression.envelope
    }
}

/// Envelope -> Expression
impl TryFrom<Envelope> for Expression {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        Ok(Self {
            function: envelope.extract_subject()?,
            envelope,
        })
    }
}

/// Envelope + optional expected function -> Expression
impl TryFrom<(Envelope, Option<&Function>)> for Expression {
    type Error = Error;

    fn try_from((envelope, expected_function): (Envelope, Option<&Function>)) -> Result<Self> {
        let expression = Expression::try_from(envelope)?;
        if let Some(expected_function) = expected_function {
            if expression.function() != expected_function {
                bail!(
                    "Expected function {:?}, but found {:?}",
                    expected_function,
                    expression.function()
                );
            }
        }
        Ok(expression)
    }
}

pub trait IntoExpression {
    fn into_expression(self) -> Expression;
    fn to_expression(&self) -> Expression;
}

impl<T: Into<Expression> + Clone + ?Sized> IntoExpression for T {
    fn into_expression(self) -> Expression {
        self.into()
    }

    fn to_expression(&self) -> Expression {
        self.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{functions, parameters};
    use indoc::indoc;

    #[test]
    fn test_expression_1() -> Result<()> {
        crate::register_tags();

        let expression = Expression::new(functions::ADD)
            .with_parameter(parameters::LHS, 2)
            .with_parameter(parameters::RHS, 3);

        let envelope: Envelope = expression.clone().into();

        let expected = indoc! {r#"
        «add» [
            ❰lhs❱: 2
            ❰rhs❱: 3
        ]
        "#}.trim();
        assert_eq!(envelope.format(), expected);

        let parsed_expression = Expression::try_from(envelope)?;

        assert_eq!(
            parsed_expression.extract_object_for_parameter::<i32>(parameters::LHS)?,
            2
        );
        assert_eq!(
            parsed_expression.extract_object_for_parameter::<i32>(parameters::RHS)?,
            3
        );

        assert_eq!(parsed_expression.function(), expression.function());
        assert_eq!(parsed_expression.expression_envelope(), expression.expression_envelope());
        assert_eq!(expression, parsed_expression);

        Ok(())
    }

    #[test]
    fn test_expression_2() -> Result<()> {
        crate::register_tags();

        let expression = Expression::new("foo")
            .with_parameter("bar", "baz")
            .with_optional_parameter("qux", None::<&str>);

        let envelope: Envelope = expression.clone().into();

        let expected = indoc! {r#"
        «"foo"» [
            ❰"bar"❱: "baz"
        ]
        "#}.trim();
        assert_eq!(envelope.format(), expected);

        let parsed_expression = Expression::try_from(envelope)?;

        assert_eq!(
            parsed_expression.extract_object_for_parameter::<String>("bar")?,
            "baz"
        );
        assert_eq!(
            parsed_expression.extract_optional_object_for_parameter::<i32>("qux")?,
            None
        );

        assert_eq!(parsed_expression.function(), expression.function());
        assert_eq!(parsed_expression.expression_envelope(), expression.expression_envelope());
        assert_eq!(expression, parsed_expression);

        Ok(())
    }
}
