#@ except: DescriptionMissing, RuntimeSectionKeys
#@ except: DisallowedInputName, MissingRequirements

## This is a test for checking for missing and extraneous entries
## in a `parameter_meta` section, and for ensuring that
## the order is the same as `input` section.

version 1.2

task t {
    meta {}

    parameter_meta {
        matching: {
            help: "a matching parameter!",
            foo: {
                bar: {
                    does_not_exist: "this should not suppress a missing input lint"
                },
            },
        }
        extra: "this should not be here"
    }

    input {
        String matching
        String does_not_exist
    }

    command <<<>>>

    output {}
}

workflow w {
    meta {}

    parameter_meta {
        matching: {
            help: "a matching parameter!",
            foo: {
                bar: {
                    does_not_exist: "this should not suppress a missing input lint"
                },
            },
        }
        extra: "this should not be here"
    }

    input {
        String matching
        String does_not_exist
    }

    output {}
}

# Task with out-of-order parameter_meta
task new_test {
    meta {}

    parameter_meta {
        second: "This should be second"
        first: "This should be first"
    }

    input {
        # This should warn about incorrect ordering
        String first
        String second
    }

    command <<<>>>

    output {}
}

struct Bar {
    meta {}

    parameter_meta {
        param_b: "This should be after param_a"
        param_a: "This should be first"
    }

    String param_a
    String param_b
}

# This should trigger a InputSorting diagnostic,
# but not a `MatchingParameterMeta` diagnostic
task input_sorting_test_1 {
    meta {}

    parameter_meta {
        b: "Another file input"
        p: "Array of non-optional strings"
        q: "Another array of non-optional strings"
        t: "File input"
        w: "Directory input"
    }

    input {
        File b
        Array[String]+ p
        Array[String]+ q
        File t
        Directory w
    }

    command <<<>>>

    output {}
}

# This should trigger both an InputSorting diagnostic
# as well as a `MatchingParameterMeta` diagnostic
task input_sorting_test_2 {
    meta {}

    parameter_meta {
        p: "Array of non-optional strings"
        w: "Directory input"
        b: "Another file input"
        q: "Another array of non-optional strings"
        t: "File input"
    }

    input {
        # Incorrect order for both input order and parameter_meta
        Directory w
        Array[String]+ p
        File t
        Array[String]+ q
        File b
    }

    command <<<>>>

    output {}
}
