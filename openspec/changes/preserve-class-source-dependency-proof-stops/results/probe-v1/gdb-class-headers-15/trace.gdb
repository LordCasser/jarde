set debuginfod enabled off
set pagination off
set confirm off
set breakpoint pending off
set print thread-events off
set disable-randomization off
break '<jarde_reader::budget::Budget>::exceeded'
commands
silent
printf "JARDE_QUOTA_REFUSAL\n"
bt 30
continue
end
run class-source --input /home/lordcasser/.codex/worktrees/core-integrity/jarde/openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/hier-fam.jar --class H1 --evidence essential --format json --budget elapsed_millis=10000 --budget output_bytes=1048576 --budget class_headers=15 > /home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1/gdb-class-headers-15/inferior.stdout.raw 2> /home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1/gdb-class-headers-15/inferior.stderr.raw
