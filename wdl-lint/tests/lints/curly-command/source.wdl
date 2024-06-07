# This is a test of the `NoCurlyCommands` lint

version 1.1

task bad {
    runtime {}
    command {}
}

task good {
    runtime {}
    command <<<>>>
}
