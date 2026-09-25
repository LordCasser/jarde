// jarde: presentation of `TernaryValues` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TernaryValues extends java.lang.Object {
    private static int trace;

    private static boolean failA;

    private static boolean failB;

    public TernaryValues() {
        // @method <init>()V
        // @declaration a constructor of `TernaryValues`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void reset(boolean arg0, boolean arg1) {
        // @method reset(ZZ)V
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = 0;
        TernaryValues.failA = arg0;
        TernaryValues.failB = arg1;
        return;
    }

    public static int trace() {
        // @method trace()I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return TernaryValues.trace;
    }

    private static int a() {
        // @method a()I
        // @declaration a static method of `TernaryValues`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = TernaryValues.trace * 10 + 1;
        if (TernaryValues.failA) {
            throw new java.lang.IllegalStateException("a");
        } else {
            return 7;
        }
    }

    private static int b() {
        // @method b()I
        // @declaration a static method of `TernaryValues`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = TernaryValues.trace * 10 + 2;
        if (TernaryValues.failB) {
            throw new java.lang.IllegalArgumentException("b");
        } else {
            return 11;
        }
    }

    public static int returned(boolean arg0) {
        // @method returned(Z)I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? a() : b();
    }

    public static int assigned(boolean arg0) {
        // @method assigned(Z)I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = arg0 ? a() : b();
        return local1;
    }

    public static int arithmetic(boolean arg0) {
        // @method arithmetic(Z)I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 2 * (arg0 ? a() : b()) + 1;
    }

    private static int add(int arg0, int arg1) {
        // @method add(II)I
        // @declaration a static method of `TernaryValues`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = TernaryValues.trace * 10 + 3;
        return arg0 + arg1;
    }

    public static int callArgument(boolean arg0) {
        // @method callArgument(Z)I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return add(100, arg0 ? a() : b());
    }

    public static java.lang.String reference(boolean arg0) {
        // @method reference(Z)Ljava/lang/String;
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? null : "value";
    }

    public static java.lang.String overloadChoice(boolean arg0) {
        // @method overloadChoice(Z)Ljava/lang/String;
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return overload(arg0 ? null : "value");
    }

    public static java.lang.String overload(java.lang.String arg0) {
        // @method overload(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = TernaryValues.trace * 10 + 4;
        return "string";
    }

    public static java.lang.String overload(java.lang.Object arg0) {
        // @method overload(Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        TernaryValues.trace = TernaryValues.trace * 10 + 5;
        return "object";
    }

    public static int throwing(boolean arg0) {
        // @method throwing(Z)I
        // @declaration a static method of `TernaryValues`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? a() : b();
    }
}
