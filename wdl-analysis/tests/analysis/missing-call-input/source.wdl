#@ except: UnusedInput, UnusedDeclaration, UnusedCall
## This is a test of a missing required call inputs.

version 1.1

task my_task {
    input {
        # Required
        String required
        # Optional
        String? optional
        # Defaulted
        String defaulted = "default"
    }

    command <<<>>>
}

workflow my_workflow {
    # Missing required input
    call my_task

    # OK
    call my_task as my_task2 { input: required = "required" }

    # OK
    call my_task as my_task3 { input: required = "required", optional = "optional", defaulted = "defaulted" }
}
