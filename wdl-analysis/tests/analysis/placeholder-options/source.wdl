version 1.2

task valid_case {
    input {
        Array[String] arr
    }
    command <<< ~{sep=", " arr} >>>
}

task invalid_case {
    command <<< ~{sep=", " true} >>>
}
