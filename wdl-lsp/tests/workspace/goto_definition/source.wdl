version 1.2

import "lib.wdl" as lib

task greet {
    input {
        String to
    }

    command <<<
        echo "Hello, ~{to}"
    >>>
}

workflow main {
    input {
        String name = "world"
    }

    #@ except: UnusedCall
    call greet { input: to = name }

    call lib.add as t1 { input:
        a = 1,
        b = 2,
    }

    Person p = Person {
        name: "test",
        age: 1,
    }

    call lib.process { input: person = p }

    #@ except: UnusedDeclaration
    String p_name = p.name

    output {
        Int result = t1.result
    }
}
