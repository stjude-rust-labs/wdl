## This is a test of having mixed indentation in commands

version 1.1

task test1 {
    command <<<
        this line is prefixed with ~{"spaces and has tailing mixed indentation"}  		
		this line is prefixed with ~{"tabs"}
      	this line is prefixed with mixed indentation
        this line is prefixed with spaces and has trailing mixed indentation  		
			   >>>

    runtime {}
}

task test2 {
    command {
        this line is prefixed with ${"spaces and has tailing mixed indentation"}  		
		this line is prefixed with ~{"tabs"}
      	this line is prefixed with mixed indentation
        this line is prefixed with spaces and has trailing mixed indentation  		
			   }

    runtime {}
}
