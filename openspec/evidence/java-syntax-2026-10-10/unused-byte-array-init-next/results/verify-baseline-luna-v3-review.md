# Unused byte-array initializer baseline verifier v3

v3 preserves v2 and corrects one representation mismatch in source-map coverage: `facts[leg]["methods"][identity]` is stored as a sorted JSON-ready list, while observed mapped BCIs are a set. The equality now converts the expected list to a set. This retains the exact coverage requirement; it does not weaken or skip BCI checks.

I reviewed the other set/list comparisons in the verifier: member identities and indices compare sets to sets; raw method BCI facts are compared as sorted lists on both sides; direct javap declarations compare lists; class/case/source inventories compare sets. No other set-vs-list equality mismatch was found. v3 is only syntax-checked and has not been executed.
