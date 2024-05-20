# This is a test of input sections in tasks and workflows.

version 1.1

task t {
    input {
        String a
        Integer b = 1 + 2
        String c = "Hello, ~{a}"
        Map[String, Integer] d
    }
}

workflow w {
    input {
        String a
        Integer b = 1 + 2
        String c = "Hello, ~{a}"
        Map[String, Integer] d
    }
}
