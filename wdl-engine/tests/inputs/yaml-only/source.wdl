## This is a test of passing inputs using YAML format only.
## No error should be present in `error.txt`.
## This test uses inputs.yaml and does not have an inputs.json file.

version 1.2

task simple_task {
    input {
        String message
        Int number
        Boolean flag
        Float value
    }

    command <<<
        echo "~{message} ~{number} ~{flag} ~{value}"
    >>>

    output {
        String result = stdout()
    }
}

workflow test {
    input {
        String message
        Int number
        Boolean flag
        Float value
    }

    call simple_task {
        input:
            message = message,
            number = number,
            flag = flag,
            value = value
    }

    output {
        String result = simple_task.result
    }
} 