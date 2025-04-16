version 1.2

task sep_option_to_function {
  input {
    Array[String] str_array
    Array[Int] int_array
  }

  command <<<>>>
  
  output {
    Boolean is_true1 = "~{sep(' ', str_array)}" == "~{sep=' ' str_array}"
    Boolean is_true2 = "~{sep(',', quote(int_array))}" == "~{sep=',' quote(int_array)}"
  }
}
