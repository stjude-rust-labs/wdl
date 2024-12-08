#@ except: DescriptionMissing, RuntimeSectionKeys

## This is a test of having no shellcheck errors

version 1.1

task test1 {

    input {
      String foo = "hello"
    }

    meta {}

    parameter_meta {}

    String bar = "there"

    command <<<
      set -eo pipefail
      
      echo ~{hello} ~{there}
    >>>

    output {}

    runtime {}
}

task test2 {
    input {
      String foo = "hello"
    }

    meta {}

    parameter_meta {}

    String bar = "there"

    #@ except: NoCurlyCommands
    command {
      set -eo pipefail

      echo ~{hello} ~{there}
    }

    output {}

    runtime {}
}
