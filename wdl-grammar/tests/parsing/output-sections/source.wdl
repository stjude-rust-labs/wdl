# This is a test of output sections in tasks and workflows.

version 1.1

task t {
    output {
        String a = "friend"
        Integer b = 1 + 2
        String c = "Hello, ~{a}"
        Map[String, Integer] d = { "a": 0, "b": 1, "c": 2}
    }
}

workflow w {
    output {
        String a = "friend"
        Integer b = 1 + 2
        String c = "Hello, ~{a}"
        Map[String, Integer] d = { "a": 0, "b": 1, "c": 2}
    }
}
