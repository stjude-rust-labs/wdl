version 1.2

#@ except: MissingRequirements, SnakeCase, NonmatchingOutput, InputSorting, VersionFormatting, Whitespace, EndingNewline

task test_declaration_names {
    meta {
        description: "This is a test of disallowed declaration names"
    }

    parameter_meta {
        fileInput: "Not OK - has type suffix"
        gtfFile: "Not OK - has type prefix"
        my_int: "Not OK - has type suffix"
        stringValue: "Not OK - has type suffix"
        booleanFlag: "Not OK - has type suffix"
        floatNumber: "Not OK - has type suffix"
        arrayData: "Not OK - has type suffix"
        validName: "OK"
        reference: "OK"
        genome: "OK"
        count: "OK"
    }

    input {
        # Invalid declarations with type prefixes/suffixes
        File fileInput
        File gtfFile
        Int my_int
        String stringValue
        Boolean booleanFlag
        Float floatNumber
        Array[Int] arrayData

        # Valid declarations
        File validName
        File reference
        String genome
        Int count
    }

    # Private declarations with type prefixes/suffixes
    File privateFile = "sample.txt"
    Int count_int = 42
    String nameString = "test"

    command <<< >>>

    output {
        # Output declarations with type prefixes/suffixes
        File outputFile = "output.txt"
        Int result_int = 42
        String resultString = "result"
    }

    runtime {}
}