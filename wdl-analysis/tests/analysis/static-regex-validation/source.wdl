version 1.2

workflow static_regex_validation {
    input {
        String s = "hello world 123"
    }

    Boolean m_valid = matches(s, "h\\w+o")  # Valid pattern

    String? f_invalid = find(s, "[unclosed")  # Invalid pattern

    output {
        Boolean result_m = m_valid
        String? result_f = f_invalid
    }
}
