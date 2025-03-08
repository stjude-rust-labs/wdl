#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sep_with_array() {
        let expr_type = Type::Array(Box::new(Type::Int)); // Array[Int]
        let placeholder = Placeholder::Sep(",".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_ok());
    }

    #[test]
    fn test_sep_with_non_array() {
        let expr_type = Type::Int; // Not an array
        let placeholder = Placeholder::Sep(",".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_err());
    }

    #[test]
    fn test_default_with_optional() {
        let expr_type = Type::Optional(Box::new(Type::String)); // Optional[String]
        let placeholder = Placeholder::Default("default_value".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_ok());
    }

    #[test]
    fn test_default_with_non_optional() {
        let expr_type = Type::String; // Not optional
        let placeholder = Placeholder::Default("default_value".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_err());
    }

    #[test]
    fn test_true_false_with_boolean() {
        let expr_type = Type::Boolean;
        let placeholder = Placeholder::TrueFalse("yes".to_string(), "no".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_ok());
    }

    #[test]
    fn test_true_false_with_non_boolean() {
        let expr_type = Type::String; // Not a boolean
        let placeholder = Placeholder::TrueFalse("yes".to_string(), "no".to_string());
        assert!(check_placeholder(&expr_type, &placeholder).is_err());
    }
}
