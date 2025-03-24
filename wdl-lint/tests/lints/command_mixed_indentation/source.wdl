version 1.0

workflow test_workflow {
    call task1
}

task task1 {
    command <<
        echo "Test line with spaces"
	    echo "Test line with tabs"
    >>
}
