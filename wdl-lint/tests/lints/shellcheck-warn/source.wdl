#@ except: DescriptionMissing, RuntimeSectionKeys, MatchingParameterMeta, NoCurlyCommands

## This is a test of having shellcheck warnings

version 1.1

task test1 {
    meta {}

    parameter_meta {}

    input {
      Int placeholder
    }

    command <<<
      somecommand.py $line17 ~{placeholder}
      somecommand.py ~{placeholder} $line18
      somecommand.py ~{placeholder}$line19










      somecommand.py $line30~{placeholder}
      somecommand.py [ -f $line31 ] ~{placeholder}
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

    command <<<
      somecommand.py $line49 ~{placeholder}
      somecommand.py ~{placeholder} $line50
      somecommand.py ~{placeholder}$line51
      somecommand.py $line52~{placeholder}
      somecommand.py [ -f $bad_test ] ~{placeholder}
      somecommand.py [ -f $trailing_space ] ~{placeholder}  
    >>>

    output {}

    runtime {}
}
