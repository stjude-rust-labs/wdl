version 1.2

task write_file {
    input {
        String content
        String file_name
    }

    command <<<
        echo "~{content}" > "~{file_name}"
    >>>

    output {
        File out_file = "~{file_name}"
    }

    runtime {
        container: "ubuntu:latest"
    }
}

task check_owner {
    input {
        File file
    }

    command <<<
        stat -c "%u" "~{file}" | grep -q $(id -u)
    >>>

    output {
        Int? code = task.return_code
    }
}

workflow chown {
    call write_file { input:
        content = "hello world!",
        file_name = "file.txt",
    }

    call check_owner { input: file = write_file.out_file }

    output {
        File result = write_file.out_file
        Boolean is_owner = check_owner.code == 0
    }
}
