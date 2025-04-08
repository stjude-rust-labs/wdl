#@ except: DescriptionMissing, RuntimeSectionKeys

version 1.1

task test1 {
    meta {}

    parameter_meta {}

    command <<<
        # This command section has consistent indentation with spaces
        echo "Hello World"
        echo "This is a command"
        echo "With consistent indentation"
    >>>

	output {
		String result = "test"
	}

    runtime {}
}

task test2 {
	meta {}

    parameter_meta {}

	command {
		# This command section has consistent indentation with tabs
		echo "Hello World"
		echo "This is a command"
		echo "With consistent indentation"
	}

    output {
        String result = "test"
    }

	runtime {}
}

workflow mixed_indentation {
    call test1
	call test2
} 