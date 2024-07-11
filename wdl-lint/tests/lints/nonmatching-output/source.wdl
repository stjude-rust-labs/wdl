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

task qux {
    meta {
        outputs: {
            t: "t",
            s: "s",
        }
    }
    command <<< >>>
    output {
        String s = "hello"
        String t = "world"
    }
}

task quux {
    meta {
        outputs: {
            s: "s",
            t: "t",
            v: "v"
        }
    }
    command <<< >>>
    output {
        String s = "hello"
        String t = "world"
    }
}

task corge {
    meta {
        outputs: "string"
    }
    command <<< >>>
    output {
        String s = "hello"
        String t = "world"
        String v = "!"
    }
}
