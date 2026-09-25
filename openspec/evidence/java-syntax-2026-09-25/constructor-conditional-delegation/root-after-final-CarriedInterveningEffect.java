// jarde: presentation of `CarriedInterveningEffect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CarriedInterveningEffect extends java.lang.Object {
    private static final java.lang.StringBuilder TRACE;

    private final java.lang.String first;

    private final java.lang.String middle;

    private final java.lang.String second;

    private static java.lang.String touch() {
        // @method touch()Ljava/lang/String;
        // @declaration a static method of `CarriedInterveningEffect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        CarriedInterveningEffect.TRACE.append("X");
        return "middle";
    }

    public CarriedInterveningEffect(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `CarriedInterveningEffect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 10
        // the carried argument crosses an independent instruction at BCI 12
        // @bytecode 12 15 16 19 21 24
        // the carried argument crosses an independent instruction at BCI 12
        // @bytecode 26 0
        // the value at BCI 26 is the entry state of stack depth 1, which no instruction produced
        return;
    }

    private CarriedInterveningEffect(java.lang.String arg1, java.lang.String arg2, java.lang.String arg3) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `CarriedInterveningEffect`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = arg1;
        this.middle = arg2;
        this.second = arg3;
        return;
    }

    public java.lang.String values() {
        // @method values()Ljava/lang/String;
        // @declaration an instance method of `CarriedInterveningEffect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append((java.lang.String) CarriedInterveningEffect.TRACE.toString()).append(":").append(this.first).append(":").append(this.middle).append(":").append(this.second).toString();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `CarriedInterveningEffect`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        TRACE = new java.lang.StringBuilder();
    }
}
