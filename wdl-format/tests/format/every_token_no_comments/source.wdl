version
1.0
workflow
test_wf
{
input
{
SpliceJunctionMotifs out_sj_filter_overhang_min = SpliceJunctionMotifs {
noncanonical_motifs: 30,
GT_AG_and_CT_AC_motif: 12,
GC_AG_and_CT_GC_motif: 12,
AT_AC_and_GT_AT_motif: 12
}
}
parameter_meta
{
out_sj_filter_overhang_min: {
type: "SpliceJunctionMotifs",
label: "Minimum overhang required to support a splicing junction"
}
}
output
{
SpliceJunctionMotifs KAZAM = out_sj_filter_overhang_min
}
meta
{
description: "Test workflow"
}

call test_task as foo {
input: bowchicka = "wowwow"
}
if (
true
) {

call test_task after foo {
input: bowchicka = "bowchicka"
}
scatter (i in range(3)) {
call test_task as bar {
input: bowchicka = i * 42
}
}
}

}
task
test_task
{
command <<<>>>
input {
String bowchicka
}
parameter_meta {
bowchicka: {
type: "String",
label: "Bowchicka"
}
}
}

struct SpliceJunctionMotifs {
Int noncanonical_motifs
Int GT_AG_and_CT_AC_motif
Int GC_AG_and_CT_GC_motif
Int AT_AC_and_GT_AT_motif
}