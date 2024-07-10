#@ except: DescriptionMissing,RuntimeSectionKeys
##
## This is a test of the `DeprecatedPlaceholderOption` lint.
##
## None of these lints should trigger as the version is WDL v1.0 (prior to
## placeholder options being deprecated).

version 1.0

task a_failing_task {
    meta {}

    String bad_sep_option = "~{sep="," numbers}"
    String bad_true_false_option = "~{true="--enable-foo" false="" allow_foo}"
    String bad_default_option = "~{default="false" bar}"

    command <<<
        python script.py ~{sep=" " numbers}
        example-command ~{true="--enable-foo" false="" allow_foo}
        another-command ~{default="foobar" bar}
    >>>

    output {}
    runtime {}
}
