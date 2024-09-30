## TODO: A strange one: this emits two errors, both from MissingRequiremnts.

version 1.2

task a_task_with_no_keys {
    #@ except: DescriptionMissing
    meta {}

    command <<<>>>

    output {}

    runtime {}
}
