version 1.2

task file_output {
  input {
    String prefix
  }

  command <<<
    printf "hello" > ~{prefix}.hello
    printf "goodbye" > ~{prefix}.goodbye
  >>>

  output {
    Array[String] basenames = [basename("~{prefix}.hello"), basename("~{prefix}.goodbye")]
  }
}
