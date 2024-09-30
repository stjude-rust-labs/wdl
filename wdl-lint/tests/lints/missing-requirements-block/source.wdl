#@ except: DescriptionMissing, Todo

version 1.2

task bad {
    meta {}

    command <<<>>>

    output {}
}

task good {
    meta {}

    command <<<>>>

    output {}

    requirements {
    }
}

# TODO: This emits two diagnostics but should only emit one.
task deprecated_runtime {
    meta {}

    command <<<>>>

    output {}

    # This `runtime` section should be flagged as deprecated.
    runtime {
    }
}
