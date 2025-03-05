version 1.0

workflow redundant_none_test {
    input {
        String required_str
        String? optional_str = None  # should flag, redundant None for optional
        Int required_int = 5  # should not flag
        Int? optional_int  # should not flag, correct optional syntax
        Float? optional_float = 3.14  # should not flag, has non-None default
    }

    call test_task {
        input:
            req_param = required_str,
            opt_param = optional_str
    }
}

task test_task {
    input {
        String req_param
        String? opt_param = None  # should flag, redundant None
    }

    command <<<  
        echo "Testing redundant None detection"
    >>>

    output {
        String result = "done"
    }
}
