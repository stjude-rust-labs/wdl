#@ except: DescriptionMissing, MissingRuntime

version 1.1

task foo {
    meta {
        outputs: {
            out: "String output of task foo"
        }
    }
    command <<< >>>
    output {
        String out = read_string(stdout())
    }
}

task bar {
    meta {}
    command <<< >>>
    output {
        String s = "hello"
    }
}

task baz {
    meta {
        outputs: {
            s: "String output of task baz"
        }
    }
    command <<< >>>
    output {
        String s = "hello"
        String t = "world"
    }
}
