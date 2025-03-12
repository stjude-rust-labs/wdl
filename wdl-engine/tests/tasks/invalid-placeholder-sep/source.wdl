version 1.2

task test {
    command<<<>>>
    output {
        String a = "~{sep=',' read_json('not-array.json')}"
    }
}
