version 1.2

# Test cases for placeholder options analysis

task valid_case {
    input {
        Array[String] arr
    }
    command <<< ~{sep=", " arr} >>> # OK: sep option with an array
}

task invalid_case {
    command <<< ~{sep=", " true} >>> # NOT OK: sep expects an array, but got Boolean
}

workflow placeholder_options_test {
    # Testing valid and invalid placeholder options

    String s1 = "~{true}" 
    # OK: Boolean is coercible to string, no option

    String s2 = "~{true="yes" false="no" false}"
    # OK: truefalse option with Boolean

    String s3 = "~{true="yes" false="no" 1}" 
    # NOT OK: truefalse expects Boolean, but got Int

    String s4 = "~{sep=',' [1, 2, 3]}" 
    # OK: sep option with Array[Int]

    String s5 = "~{sep=',' 123}" 
    # NOT OK: sep expects Array, but got Int

    String s6 = "~{default="fallback" var?}" 
    # OK: default option with optional variable

    String s7 = "~{default="fallback" 123}" 
    # NOT OK: default expects optional, but got Int

    String s8 = "~{[1, 2, 3]}" 
    # NOT OK: Array without sep option, not coercible to string
}
