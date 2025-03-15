#@ except: DescriptionMissing, RuntimeSectionKeys

## This is a test for checking for missing and extraneous entries
## in a `parameter_meta` section, and for ensuring that
## the order is the same as `input` section.

version 1.1

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
        # This should warn about missing parameter metadata but not the order
        String matching
        String does_not_exist
    }

    command <<<>>>

    output {}

    runtime {}
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

workflow test {
    meta {}

    parameter_meta {
        one: "first"
        two: "second"
    }

    input {
        # This should warn about incorrect ordering
        String two
        String one
    }

    output {}
}
