# This is a test for mixed indentation in command sections.

version 1.1

task test {
    command <<<
        echo "This line is indented with tabs"
        echo "This line is indented with spaces"
	    echo "This line has mixed indentation"
    >>>
    
    output {
        String out = stdout()
    }
} 