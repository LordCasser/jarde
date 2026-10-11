# Hoisted iterator SSA boundaries

Java 8 fixture: mixedIterator merges the initial and a replacement iterator in the same local; extraAliasConsumer copies the original into a branch-written alias. Both are refusal controls for the production iterator use gate, not foreach positives.

Frozen canonical class: Corretto javac 1.8.0_432, `-source 8 -target 8 -Xlint:-options -proc:none -g:none`, empty classpath/sourcepath, no annotation processing. Source 707B SHA256 9cdf3915818be01928664d3c112b3f846797b9f71d110dc49df8d8144bd59bbb; class606B SHA256 31cd1bcbcd906ac5249ba6b917660774658b157964451136bbb3b86bc694ccda.

The same source was compiled with OpenJDK23.0.1 `--release 8` under the 5GiB free/target1GiB process-group guard. That output is603B SHA256 87e300ec06fdc83dafc8e3cceb3ff5c2e3dc6300dfad6e794451061f5348f2a4. The outputs have the same method BCIs but are not byte-identical. Actual argv/raw javap/class pins are archived in the OpenSpec results. No Java runtime equivalence claim is made for these SSA-only controls.

Copying a local uses the original value at load11 and defines a fresh stack/local value. Do not claim the original store6 ValueId is an exact input of the alias slot4 Phi. The extra physical load is the actual refusal boundary.
