# This is a test for mixed indentation in the document.

version 1.1

workflow test_workflow {
    input {
        String message
    }
	
	# This line is indented with tabs
    
    call echo_task {
        input:
            message = message
    }
	
	# Another line with tab indentation
    
    output {
        String out = echo_task.out
    }
}

task echo_task {
    input {
        String message
    }
    
    command <<<
        echo "${message}"
    >>>
    
	output {
        String out = stdout()
    }
} 