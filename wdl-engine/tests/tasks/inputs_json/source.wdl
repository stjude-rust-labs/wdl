version 1.2

task write_inputs_test {
  input {
    String message = "hello"
    Int number = 42
    Boolean flag = true
  }

  command {
    echo "~{message} ~{number} ~{flag}"
  }

  output {
    String result = stdout()
  }
}