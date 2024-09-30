#@ except: DescriptionMissing, Todo

## This is a test of a missing blank line following the version statement.
## TODO: this emits two errors, one from VersionFormatting and one from
## BlankLinesBetweenElements. Only one of the errors is expected.

version 1.1
workflow test {
    meta {}

    output {}
}
