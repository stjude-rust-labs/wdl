version 1.2

task t {
    input {
        File remote1
        File remote2
        File remote3
        File remote4
        File remote5
        File remote6
        File remote7
        File remote8
        File remote9
        File remote10
        File local_file
    }

    File relative_path = "relative.txt"

    command <<<
        set -euo pipefail
        cat '~{remote1}' > remote1
        cat '~{remote2}' > remote2
        cat '~{remote3}' > remote3
        cat '~{remote4}' > remote4
        cat '~{remote5}' > remote5
        cat '~{remote6}' > remote6
        cat '~{remote7}' > remote7
        cat '~{remote8}' > remote8
        cat '~{remote9}' > remote9
        cat '~{remote10}' > remote10
        cat '~{local_file}' > ~{relative_path}
    >>>

    output {
        File out1 = "remote1"
        File out2 = "remote2"
        File out3 = "remote3"
        File out4 = "remote4"
        File out5 = "remote5"
        File out6 = "remote6"
        File out7 = "remote7"
        File out8 = "remote8"
        File out9 = "remote9"
        File out10 = "remote10"
        File relative_out = relative_path
    }
}

workflow test {
    input {
        File remote1
        File remote2
        File remote3
        File remote4
        File remote5
        File remote6
        File remote7
        File remote8
        File remote9
        File remote10
        File local_file
    }

    call t { input:
        remote1,
        remote2,
        remote3,
        remote4,
        remote5,
        remote6,
        remote7,
        remote8,
        remote9,
        remote10,
        local_file,
    }

    output {
        Object out1 = read_json(t.out1)
        Object out2 = read_json(t.out2)
        Object out3 = read_json(t.out3)
        Object out4 = read_json(t.out4)
        Object out5 = read_json(t.out5)
        Object out6 = read_json(t.out6)
        Object out7 = read_json(t.out7)
        Object out8 = read_json(t.out8)
        Object out9 = read_json(t.out9)
        Object out10 = read_json(t.out10)
        String relative_out = read_string(t.relative_out)
    }
}
