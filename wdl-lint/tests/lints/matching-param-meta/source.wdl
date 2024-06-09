## This is a test for checking for missing and extraneous entries in a `parameter_meta` section.

version 1.1

task t {
    input {
        String matching
        String does_not_exist
    }

    parameter_meta {
        matching: {
            help: "a matching parameter!"
        }
        extra: "this should not be here"
    }

    runtime {}
    command <<<>>>
}

workflow w {
    input {
        String matching
        String does_not_exist
    }

    parameter_meta {
        matching: {
            help: "a matching parameter!"
        }
        extra: "this should not be here"
    }
}
