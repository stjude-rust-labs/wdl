#@ except: DescriptionMissing, InputSorting, MissingMetas, MissingOutput, MissingRuntime

version 1.1

import "baz"
import "qux"


workflow foo {
    meta {}
    parameter_meta {}
    input {}

    String s = "hello"

    call bar { input:
        s = s
    }


    call bar as baz { input:
        s = s
    }
}
task bar {

    meta {
        description: "bar"

        outputs: {
            u: "u"

        }
    }

    input {
        String s = "hello"

        String? t
    }

    command <<< >>>

    output {
        String u = "u"
    }

}
