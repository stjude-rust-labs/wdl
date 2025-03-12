#@ except: DescriptionMissing, MissingRequirements
#@ except: RuntimeSectionKeys, MissingMetas, MissingOutput

version 1.0

workflow redundant_none_test {
    input {
        String required_str
        Int? optional_int  # should not flag, correct optional syntax
        String? optional_str = None  # should flag, redundant None for optional
        Float? optional_float = 3.14  # should not flag, has non-None default
        Int required_int = 5  # should not flag
    }

    # Test in a task context as well
    call test_task {
        input:
            req_param = required_str,
            opt_param = optional_str,
    }
}

task test_task {
    input {
        String req_param
        Int? opt_int  # should not flag
        File? opt_file = None  # should not flag due to except directive
        String? opt_param = None  # should flag, redundant None
        Boolean? opt_bool = true  # should not flag, has non-None default
    }
    
    command <<<
        echo "Testing redundant None detection"
    >>>
    
    output {
        String result = "done"
    }
}