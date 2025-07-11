## This is a test of a failure to coerce an element of an array literal.
version 1.2

workflow test {
    Array[Int]+? a = [1, 2, 3]
    Array[Int] b = select_first([a, []])
}
