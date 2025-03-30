## This is a test for checking for missing and extraneous entries
## in a `parameter_meta` section, and for ensuring that
## the order is the same as `input` section.

version 1.2

struct Text {
    meta {
        description: "foo"
    }

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

    String matching
    String does_not_exist
}
