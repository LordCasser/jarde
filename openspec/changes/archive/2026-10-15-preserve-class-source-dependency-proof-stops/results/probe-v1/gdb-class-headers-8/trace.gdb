set pagination off
set confirm off
set breakpoint pending off
set print thread-events off
set disable-randomization off
break '<jarde_reader::budget::Budget>::exceeded'
commands
silent
printf "JARDE_QUOTA_REFUSAL\n"
bt 14
continue
end
run > /home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1/gdb-class-headers-8/inferior.stdout.raw 2> /home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1/gdb-class-headers-8/inferior.stderr.raw
