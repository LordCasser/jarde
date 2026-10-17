# Loop switch exception boundary

Original whole Java source compiled by real javac8 (-source/-target 8) and javac23 (--release 8), then both class legs verified on both JVMs with -Xverify:all. Full stdout: empty=0, normal=7, case-return=2, caught=-7, null=-7. Manifest records exact commands, class/source pins and Code facts. Tests require a real canonical exception edge incident to dispatch before strict-completion refusal. This does not require public Try recovery to fail.
