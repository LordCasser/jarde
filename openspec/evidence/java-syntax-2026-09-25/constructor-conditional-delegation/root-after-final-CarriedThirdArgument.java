// jarde: presentation of `CarriedThirdArgument` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CarriedThirdArgument extends java.lang.Object {
    private final java.lang.String first;

    private final java.lang.String second;

    private final int third;

    public CarriedThirdArgument(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `CarriedThirdArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 10
        // the carried conditional values are not consecutive invocation arguments
        // @bytecode 12 13 16 18 21
        // the carried conditional values are not consecutive invocation arguments
        // @bytecode 24 0 23
        // the value at BCI 24 is the entry state of stack depth 1, which no instruction produced
        return;
    }

    private CarriedThirdArgument(java.lang.String arg1, java.lang.String arg2, int arg3) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;I)V
        // @declaration a constructor of `CarriedThirdArgument`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = arg1;
        this.second = arg2;
        this.third = arg3;
        return;
    }

    public java.lang.String values() {
        // @method values()Ljava/lang/String;
        // @declaration an instance method of `CarriedThirdArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(this.first).append(":").append(this.second).append(":").append(this.third).toString();
    }
}
