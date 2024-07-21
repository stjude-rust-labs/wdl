#1
version
#2
1.0
#3
workflow
#4
test_wf
#5
{
input
{
SpliceJunctionMotifs
 out_sj_filter_overhang_min
  = 
   SpliceJunctionMotifs
    {
noncanonical_motifs
:
 30
 , 
GT_AG_and_CT_AC_motif 
: 
12 
, 
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
#6
meta
#7
{
#8
description
#9
:
#10
 "Test workflow"
#11
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
}