# Test SnakeCase rule

version 1.0

workflow BadWorkflow {
    call BadTask
    call good_task
}

task BadTask {
    command <<<
        echo "Hello World"
    >>>
    output {
        File badOut = "out.txt"
    }
    runtime {}
}

task good_task {
    command <<<
        echo "Hello World"
    >>>
    output {
        File good_out = "out.txt"
    }
    runtime {}
}

struct GoodStruct {
    String good_field
    String bAdFiElD  # unfortunately, `convert-case` doesn't understand sarcasm case
}
