version 1.2

import "lib.wdl"
import "lib.wdl" as lib_alias

struct Person {
    String name
    Int age
}

task greet {
    input {
        Person person
    }

    String message = "hello ~{person.name}"

    command <<<
        echo "~{message}"
    >>>

    output {
        String out = read_string(stdout())
    }
}

workflow main {
    input {
        Person p
    }

    call greet { input: person = p }

    output {
        String result = greet.out
    }
}
