# This is a test for an import with an unsupported version, but with a fallback configured. The
# result should be a successful interpretation of both documents the import as whatever the
# "current" version is (1.2 as of initially writing this test), albeit with a warning diagnostic.

version 1.0

import "foo.wdl"

workflow test {
}
