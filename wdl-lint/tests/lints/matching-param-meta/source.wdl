#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements

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
task foo {
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
