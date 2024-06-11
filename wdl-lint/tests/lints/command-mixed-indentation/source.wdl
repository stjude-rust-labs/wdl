## This is a test of having mixed indentation in command sections.

version 1.1

task test1 {
    command <<<
        this line is ~{
		    if true
		    then "split across multiple lines with mixed indentation"
		    else "by a placeholder"
	    } but is all one literal line in the command text
        this line has a continuation \
 		   and should not be a warning
        this line is prefixed with ~{"spaces and has tailing mixed indentation"}  		
		this line is prefixed with ~{"tabs"}
      	this line is prefixed with mixed indentation
        this line is prefixed with spaces and has trailing mixed indentation  		
			   >>>

    runtime {}
}

task test2 {
    command {
        this line is ~{
		    if true
		    then "split across multiple lines with mixed indentation"
		    else "by a placeholder"
	    } but is all one literal line in the command text
        this line has a continuation \
 		   and should not be a warning
        this line is prefixed with ${"spaces and has tailing mixed indentation"}  		
		this line is prefixed with ~{"tabs"}
      	this line is prefixed with mixed indentation
        this line is prefixed with spaces and has trailing mixed indentation  		
			   }

    runtime {}
}
