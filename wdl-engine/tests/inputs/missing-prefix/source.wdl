## This is a test of a missing prefix in an input key.
## No error should be emitted as the prefix should be inferred.

version 1.1

workflow test {
    input {
        Int x
        Int y
    }
}
