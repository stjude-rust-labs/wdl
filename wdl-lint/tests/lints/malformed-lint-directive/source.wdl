#@ except: DescriptionMissing, MissingRuntime, DescriptionMissing, UnknownRule

## This is a test of the `MalformedLintDirective` rule

version 1.2

#@ stop: This should be flagged for using 'stop' instead of 'except'

#@ except: MissingRequirements
task foo {
    #@except: this should be flagged for missing a space
    meta {
    }

    command <<<>>>

    output {
    }

    runtime {
    }
}

workflow bar {
    meta {
    }

    #@ except this should be flagged for missing a colon
    output {
    }
}

struct Baz {  #@ except: this should be flagged for being inlined
    String x

    meta {
    }

    parameter_meta {
        x: "foo"
    }
}
