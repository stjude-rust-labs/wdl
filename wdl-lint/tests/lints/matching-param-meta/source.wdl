#@ except: DescriptionMissing, RuntimeSectionKeys

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
        # This should warn about missing parameter metadata
        String matching
        String does_not_exist
    }

    command <<<>>>

    output {}

    requirements {}
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
        # This should warn about extra parameter metadata
        String does_not_exist
        String matching
    }

    output {}
}

# Task with out-of-order parameter_meta
task new_test {
    meta {}

    parameter_meta {
        first: "This should be second"
        second: "This should be first"
    }

    input {
        # This should warn about incorrect ordering
        String second
        String first
    }

    command <<<>>>

    output {}

    requirements {}
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
