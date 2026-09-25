// jarde: presentation of `CarriedNonPrologue` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CarriedNonPrologue extends java.lang.Object {
    private final CarriedNonPrologue$Nested value;

    public CarriedNonPrologue(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `CarriedNonPrologue`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        // @bytecode 5
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 8
        // the instruction at BCI 8 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 0 14 18 11
        // the carried-value constructor call has no proved prologue
        // @bytecode 20 24 29 21
        // the carried-value constructor call has no proved prologue
        // @bytecode 31
        // the dependency chain from BCI 31 to final consumer 34 is not bounded
        // @bytecode 34 31 4
        // the value at BCI 34 was produced by a saved declaration this run could not commit
        return;
    }

    public java.lang.String values() {
        // @method values()Ljava/lang/String;
        // @declaration an instance method of `CarriedNonPrologue`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append((java.lang.String) CarriedNonPrologue$Nested.access$000(this.value)).append(":").append((java.lang.String) CarriedNonPrologue$Nested.access$100(this.value)).toString();
    }
}
