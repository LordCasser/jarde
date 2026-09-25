// jarde: presentation of `CarriedTypeMismatch` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CarriedTypeMismatch extends java.lang.Object {
    private final java.lang.Object first;

    private final java.lang.String second;

    public CarriedTypeMismatch(java.lang.Object arg1, boolean arg2, int arg3) {
        // @method <init>(Ljava/lang/Object;ZI)V
        // @declaration a constructor of `CarriedTypeMismatch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 5 9 2
        // the two values joined at BCI 22 do not have a conditional Java type this run can prove
        // @bytecode 11 15 20 12
        // the two values joined at BCI 22 do not have a conditional Java type this run can prove
        // @bytecode 22 0
        // the value at BCI 22 is the entry state of stack depth 1, which no instruction produced
        return;
    }

    private CarriedTypeMismatch(java.lang.Object arg1, java.lang.String arg2) {
        // @method <init>(Ljava/lang/Object;Ljava/lang/String;)V
        // @declaration a constructor of `CarriedTypeMismatch`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = arg1;
        this.second = arg2;
        return;
    }

    public java.lang.String values() {
        // @method values()Ljava/lang/String;
        // @declaration an instance method of `CarriedTypeMismatch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(this.first).append(":").append(this.second).toString();
    }
}
