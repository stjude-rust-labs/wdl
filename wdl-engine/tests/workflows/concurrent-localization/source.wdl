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
        File remote11
        File remote12
        File remote13
        File remote14
        File remote15
        File remote16
        File remote17
        File remote18
        File remote19
        File remote20
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
        cat '~{remote11}' > remote11
        cat '~{remote12}' > remote12
        cat '~{remote13}' > remote13
        cat '~{remote14}' > remote14
        cat '~{remote15}' > remote15
        cat '~{remote16}' > remote16
        cat '~{remote17}' > remote17
        cat '~{remote18}' > remote18
        cat '~{remote19}' > remote19
        cat '~{remote20}' > remote20
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
        File out11 = "remote11"
        File out12 = "remote12"
        File out13 = "remote13"
        File out14 = "remote14"
        File out15 = "remote15"
        File out16 = "remote16"
        File out17 = "remote17"
        File out18 = "remote18"
        File out19 = "remote19"
        File out20 = "remote20"
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
        File remote11
        File remote12
        File remote13
        File remote14
        File remote15
        File remote16
        File remote17
        File remote18
        File remote19
        File remote20
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
        remote11,
        remote12,
        remote13,
        remote14,
        remote15,
        remote16,
        remote17,
        remote18,
        remote19,
        remote20,
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
        Object out11 = read_json(t.out11)
        Object out12 = read_json(t.out12)
        Object out13 = read_json(t.out13)
        Object out14 = read_json(t.out14)
        Object out15 = read_json(t.out15)
        Object out16 = read_json(t.out16)
        Object out17 = read_json(t.out17)
        Object out18 = read_json(t.out18)
        Object out19 = read_json(t.out19)
        Object out20 = read_json(t.out20)
        String relative_out = read_string(t.relative_out)
    }
}
