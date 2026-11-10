# Declared return-consumption controls R1

Frozen Corretto 8 class with four return cases: a parameterized local copy, a method binder bounded by `List<Foo>`, an inherited generic field return whose original Code has exactly one `checkcast List`, and an unchecked `Object`→`List<Foo>` cast with no declared producer for its generic argument.

`ReturnConsumptionControls.javap.txt` records exact descriptors, generic Signatures, and Code. `returnField` has a single original c0 `java/util/List` at BCI 4. `unchecked` has one c0 List at BCI 1. The class was compiled against the frozen R5 Base header and child Foo. No Java method is executed.
