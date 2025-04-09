version 1.2
task write_inputs_test {
  input {
    String message = "testing inputs json"
    Int number = 100
    Boolean flag = false
  }
  command {
    echo "Test"
  }
  output {
    String out = "test"
  }
}
