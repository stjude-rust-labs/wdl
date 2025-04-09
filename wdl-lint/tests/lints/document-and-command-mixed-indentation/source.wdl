#@ except: DescriptionMissing, RuntimeSectionKeys

## This is a test of having mixed indentation both in document and command sections.

version 1.1

task test1 {
    meta {}

    parameter_meta {}

    command <<<
        this line has spaces
		this line has tabs
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
        this line has spaces
		this line has tabs
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
