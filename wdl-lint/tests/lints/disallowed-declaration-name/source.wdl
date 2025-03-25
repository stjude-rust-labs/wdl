#@ except: MissingRequirements, SnakeCase, InputSorting, NonmatchingOutput, MissingMetas

version 1.2

task test_declaration_names {
    meta {
        description: "This is a test of disallowed declaration names"
    }

    input {
        # BAD
        Array[Int] arrayData
        Boolean bool_flag
        Float floatNumber
        Int my_int
        Directory dir
        Directory reference_directory

        # GOOD
        Int intermittent
        File Interval
        String name
        String name_str
        String name_string
        Directory direct_descendant
    }

    # BAD
    Int count_int = 42
    Int result_integer = 42

    # GOOD
    String nameString = "test"

    command <<<>>>

    output {
        # BAD
        Int result_int = 42
        # GOOD
        File file = "output.txt"
        String resultString = "result"
    }

    runtime {}
}
