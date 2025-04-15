## This is a test of properly translating paths when using the `sep` placeholder option.

version 1.2

task test {
    input {
        Array[File] files
    }

    command <<<
        set -euo pipefail
        echo '~{if task.container == "ubuntu:latest" then if "~{sep=',' files}" == "/mnt/inputs/10/a.txt,/mnt/inputs/10/b.txt,/mnt/inputs/10/c.txt" then "ok!" else "bad :(" else "ok!" }'
        cat '~{files[0]}'
        cat '~{files[1]}'
        cat '~{files[2]}'
    >>>

    output {
        String out = read_string(stdout())
    }
}
