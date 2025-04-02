version 1.2

task test {
    command {
        echo "Running test..."
    }

    output {
        Array[String] json_array = read_json("not-array.json")
        String a = "~{sep=', ' json_array}" # This might still be incorrect
    }
}
