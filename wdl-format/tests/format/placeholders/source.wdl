version 1.2

task test {
  input {
    String a_string = "foo ${baz}"
    String another_string = "bar ~{
        baz
    }"
    String a_third_string = "baz ${false='no' true='yes' a_boolean}"
  }
  command <<< >>>
}