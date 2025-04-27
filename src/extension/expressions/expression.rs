use anyhow::Result;
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeEncodable, Function, Parameter};

/// An expression in a Gordian Envelope.
///
/// An expression consists of a function (the subject of the envelope) and zero
/// or more parameters (as assertions on the envelope). It represents a
/// computation or function call that can be evaluated.
///
/// Expressions form the foundation for Gordian Envelope's ability to represent
/// computations, queries, and function calls within the envelope structure.
///
/// Expressions are documented in [BCR-2023-012: Envelope
/// Expressions](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-012-envelope-expression.md).
///
/// # Examples
///
/// A simple addition expression:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Create an expression that adds 2 and 3
/// let expression = Expression::new(functions::ADD)
///     .with_parameter(parameters::LHS, 2)
///     .with_parameter(parameters::RHS, 3);
/// ```
///
/// A more complex expression:
///
/// ```ignore
/// use bc_envelope::prelude::*;
///
/// // Create a verify signature expression with a public key, signature, and digest
/// let expression = Expression::new("verifySignature")
///     .with_parameter("key", public_key)
///     .with_parameter("sig", signature)
///     .with_parameter("digest", message_digest);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The function being called in this expression
    function: Function,
    /// The envelope representing this expression
    envelope: Envelope,
}

impl Expression {
    /// Creates a new expression with the given function.
    ///
    /// The function becomes the subject of the expression envelope.
    ///
    /// # Parameters
    ///
    /// * `function` - The function identifier for this expression
    ///
    /// # Returns
    ///
    /// A new Expression instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a new expression with the ADD function
    /// let expression = Expression::new(functions::ADD);
    ///
    /// // Create a new expression with a named function
    /// let expression = Expression::new("verifySignature");
    /// ```
    pub fn new(function: impl Into<Function>) -> Self {
        let function = function.into();
        Self {
            function: function.clone(),
            envelope: Envelope::new(function),
        }
    }
}

/// Implements Display for Expression.
///
/// Outputs the formatted envelope representation of the expression.
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.envelope.format())
    }
}

/// Behavior for working with expressions.
///
/// This trait defines methods for composing expressions by adding parameters,
/// and for parsing and extracting values from expression parameters.
///
/// ExpressionBehavior is implemented by Expression and other types that
/// represent expressions in Gordian Envelope.
pub trait ExpressionBehavior {
    //
    // Composition
    //

    /// Adds a parameter to the expression.
    ///
    /// This creates an assertion on the expression envelope with the parameter
    /// as the predicate and the value as the object.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter identifier
    /// * `value` - The value for the parameter
    ///
    /// # Returns
    ///
    /// A new instance with the parameter added
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let expression = Expression::new(functions::ADD)
    ///     .with_parameter(parameters::LHS, 2)
    ///     .with_parameter(parameters::RHS, 3);
    /// ```
    fn with_parameter(self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self;

    /// Adds a parameter to the expression if the value is not `None`.
    ///
    /// If the value is `Some`, this creates an assertion on the expression envelope
    /// with the parameter as the predicate and the value as the object.
    /// If the value is `None`, the parameter is not added.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter identifier
    /// * `value` - An optional value for the parameter
    ///
    /// # Returns
    ///
    /// A new instance with the parameter added if the value is Some
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let expression = Expression::new(functions::ADD)
    ///     .with_parameter(parameters::LHS, 2)
    ///     .with_optional_parameter(parameters::RHS, Some(3))
    ///     .with_optional_parameter("note", None::<&str>);
    /// ```
    fn with_optional_parameter(self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self;

    //
    // Parsing
    //

    /// Returns the function identifier of the expression.
    ///
    /// # Returns
    ///
    /// A reference to the Function of this expression
    fn function(&self) -> &Function;

    /// Returns the envelope that represents this expression.
    ///
    /// # Returns
    ///
    /// A reference to the Envelope of this expression
    fn expression_envelope(&self) -> &Envelope;

    /// Returns the argument (object) for the given parameter.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to look up
    ///
    /// # Returns
    ///
    /// The argument envelope for the parameter, or an error if not found
    ///
    /// # Errors
    ///
    /// Returns an error if no matching parameter is found or if multiple
    /// parameters match.
    fn object_for_parameter(&self, param: impl Into<Parameter>) -> anyhow::Result<Envelope>;

    /// Returns all arguments (objects) for the given parameter.
    ///
    /// This method handles the case where a parameter appears multiple times.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to look up
    ///
    /// # Returns
    ///
    /// A vector of all matching argument envelopes, or an empty vector if none are found
    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope>;

    /// Returns the argument for the given parameter, decoded as the given type.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to look up
    ///
    /// # Returns
    ///
    /// The argument for the parameter, decoded as type T
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No matching parameter is found
    /// - Multiple parameters match
    /// - The argument cannot be decoded as type T
    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static;

    /// Returns the argument for the given parameter, decoded as the given type,
    /// or `None` if there is no matching parameter.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to look up
    ///
    /// # Returns
    ///
    /// Some(T) if the parameter is found and can be decoded, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Multiple parameters match
    /// - The argument cannot be decoded as type T
    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(
        &self,
        param: impl Into<Parameter>,
    ) -> Result<Option<T>>;

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    ///
    /// This method handles the case where a parameter appears multiple times.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to look up
    ///
    /// # Returns
    ///
    /// A vector of all matching arguments, decoded as type T
    ///
    /// # Errors
    ///
    /// Returns an error if any of the arguments cannot be decoded as type T
    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static;
}

/// Implementation of ExpressionBehavior for Expression.
impl ExpressionBehavior for Expression {
    /// Adds a parameter to the expression.
    fn with_parameter(
        mut self,
        parameter: impl Into<Parameter>,
        value: impl EnvelopeEncodable,
    ) -> Self {
        let assertion = Envelope::new_assertion(parameter.into(), value.into_envelope());
        self.envelope = self.envelope.add_assertion_envelope(assertion).unwrap();
        self
    }

    /// Adds a parameter to the expression if the value is not None.
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

    /// Returns the function of the expression.
    fn function(&self) -> &Function {
        &self.function
    }

    /// Returns the envelope representing the expression.
    fn expression_envelope(&self) -> &Envelope {
        &self.envelope
    }

    /// Returns the argument for the given parameter.
    fn object_for_parameter(&self, param: impl Into<Parameter>) -> anyhow::Result<Envelope> {
        self.envelope.object_for_predicate(param.into())
    }

    /// Returns all arguments for the given parameter.
    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.envelope.objects_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, decoded as the given type.
    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.envelope.extract_object_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, decoded as the given type,
    /// or None if there is no matching parameter.
    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = dcbor::Error> + 'static>(
        &self,
        param: impl Into<Parameter>,
    ) -> Result<Option<T>> {
        self.envelope
            .extract_optional_object_for_predicate(param.into())
    }

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.envelope.extract_objects_for_predicate(param.into())
    }
}

/// Allows converting an Expression to an Envelope.
///
/// This simply returns the envelope that represents the expression.
impl From<Expression> for Envelope {
    fn from(expression: Expression) -> Self {
        expression.envelope
    }
}

/// Allows converting an Envelope to an Expression.
///
/// This extracts the function from the envelope's subject and creates
/// an Expression with that function and the envelope.
///
/// # Errors
///
/// Returns an error if the envelope's subject cannot be extracted as a Function.
impl TryFrom<Envelope> for Expression {
    type Error = dcbor::Error;

    fn try_from(envelope: Envelope) -> dcbor::Result<Self> {
        let function = envelope.extract_subject()?;
        Ok(Self {
            function,
            envelope,
        })
    }
}

/// Allows converting an Envelope and optional expected function to an Expression.
///
/// This is similar to `TryFrom<Envelope>`, but it also checks that the function
/// matches the expected function, if provided.
///
/// # Errors
///
/// Returns an error if:
/// - The envelope's subject cannot be extracted as a Function
/// - The expected function is provided and doesn't match the extracted function
impl TryFrom<(Envelope, Option<&Function>)> for Expression {
    type Error = dcbor::Error;

    fn try_from((envelope, expected_function): (Envelope, Option<&Function>)) -> dcbor::Result<Self> {
        let expression = Expression::try_from(envelope)?;
        if let Some(expected_function) = expected_function {
            if expression.function() != expected_function {
                return Err(format!(
                    "Expected function {:?}, but found {:?}",
                    expected_function,
                    expression.function()
                ).into());
            }
        }
        Ok(expression)
    }
}

/// A trait for converting types to Expression.
///
/// This trait provides convenience methods for converting a type to
/// an Expression, either by consuming it or by cloning it.
pub trait IntoExpression {
    /// Converts this object into an Expression, consuming it.
    fn into_expression(self) -> Expression;

    /// Creates an Expression from this object, without consuming it.
    fn to_expression(&self) -> Expression;
}

/// Implementation of IntoExpression for any type that can be converted to Expression.
///
/// This allows any type that implements `Into<Expression>` to be used with the
/// convenience methods provided by the IntoExpression trait.
impl<T: Into<Expression> + Clone> IntoExpression for T {
    /// Converts this object into an Expression, consuming it.
    fn into_expression(self) -> Expression {
        self.into()
    }

    /// Creates an Expression from this object, without consuming it.
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
    fn test_expression_1() -> anyhow::Result<()> {
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
    fn test_expression_2() -> anyhow::Result<()> {
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
