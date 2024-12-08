#@ except: DescriptionMissing, RuntimeSectionKeys

## This is a test of having shellcheck warnings

version 1.1

task test1 {
    meta {}

    parameter_meta {}

    command <<<
    set -eo pipefail
    foo="123 456"
    echo $foo
    >>>

    output {}

    runtime {}
}

task test2 {
    meta {}

    parameter_meta {}

    #@ except: NoCurlyCommands
    command {
      set -eo pipefail
      foo="123 456"
      echo $foo
    }

    output {}

    runtime {}
}
