## This is a test of a failure to coerce an element of a map literal.
version 1.2

workflow test {
    Array[Int]+? a = [1, 2, 3]
    Object b = object {
        key: { "a": a, "b": [] }
    }
}
