#@ except: DescriptionMissing, RuntimeSectionKeys, MatchingParameterMeta, NoCurlyCommands

## This is a test of having no shellcheck lints

version 1.1

task test1 {
    meta {}

    parameter_meta {}

    input {
      Boolean i_quote_my_shellvars
      Int placeholder
    }

    command <<<
      set -eo pipefail

      echo "$placeholder"

      if [[ "$i_quote_my_shellvars" ]]; then
        echo "shellcheck will be happy"
      fi
    >>>

    output {}

    runtime {}
}

task test2 {
    meta {}

    parameter_meta {}

    input {
      Int placeholder
    }

    command {
      set -eo pipefail

      echo "$placeholder"

      if [[ "$I_quote_my_shellvars" ]]; then
        echo "all is well"
      fi
    }

    output {}

    runtime {}
}
