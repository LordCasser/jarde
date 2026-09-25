// jarde: presentation of `CarriedMethodCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CarriedMethodCalls extends java.lang.Object {
    private static final java.lang.StringBuilder TRACE;

    public CarriedMethodCalls() {
        // @method <init>()V
        // @declaration a constructor of `CarriedMethodCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.String mark(java.lang.String arg0, java.lang.String arg1) {
        // @method mark(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `CarriedMethodCalls`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        CarriedMethodCalls.TRACE.append(arg0);
        return arg1;
    }

    private static java.lang.String combineStatic(java.lang.String arg0, java.lang.String arg1) {
        // @method combineStatic(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `CarriedMethodCalls`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + arg1;
    }

    private java.lang.String combineInstance(java.lang.String arg1, java.lang.String arg2) {
        // @method combineInstance(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `CarriedMethodCalls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 + arg2;
    }

    public static java.lang.String staticCall(boolean arg0, boolean arg1) {
        // @method staticCall(ZZ)Ljava/lang/String;
        // @declaration a static method of `CarriedMethodCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        CarriedMethodCalls.TRACE.setLength(0);
        java.lang.String local2 = combineStatic(arg0 ? mark("A", "a") : mark("B", "b"), arg1 ? mark("C", "c") : mark("D", "d"));
        return new java.lang.StringBuilder().append((java.lang.String) CarriedMethodCalls.TRACE.toString()).append(":").append(local2).toString();
    }

    public java.lang.String instanceCall(boolean arg1, boolean arg2) {
        // @method instanceCall(ZZ)Ljava/lang/String;
        // @declaration an instance method of `CarriedMethodCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        CarriedMethodCalls.TRACE.setLength(0);
        java.lang.String local3 = this.combineInstance(arg1 ? mark("A", "a") : mark("B", "b"), arg2 ? mark("C", "c") : mark("D", "d"));
        return new java.lang.StringBuilder().append((java.lang.String) CarriedMethodCalls.TRACE.toString()).append(":").append(local3).toString();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `CarriedMethodCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        TRACE = new java.lang.StringBuilder();
    }
}
