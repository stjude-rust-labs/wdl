## This is a test of import placements.

version 1.1

import "good"
import "good"
import "good"

workflow test {
    meta {}
    output {}
}

import "bad"
import "also bad"
