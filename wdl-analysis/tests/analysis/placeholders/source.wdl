# This is a test for analysis of placeholders and their options.
version 1.1

workflow test {
    String s1 = "~{true}" # OK: Boolean is coercible to string, no option
    String s2 = "~{true="yes" false="no" false}" # OK: truefalse option with Boolean
    String s3 = "~{true="yes" false="no" 1}" # NOT OK: truefalse expects Boolean, but got Int
    String s4 = "~{sep=',' [1, 2, 3]}" # OK: sep option with Array[Int]
    String s5 = "~{sep=',' 123}" # NOT OK: sep expects Array, but got Int
    String s6 = "~{default="fallback" var?}" # OK: default option with optional variable
    String s7 = "~{default="fallback" 123}" # NOT OK: default expects optional, but got Int
    String s8 = "~{[1, 2, 3]}" # NOT OK: Array without sep option, not coercible to string
}