// jarde: presentation of `ShortCircuitFalse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class ShortCircuitFalse extends java.lang.Object {
    static boolean result;

    static int calls;

    ShortCircuitFalse() {
        // @method <init>()V
        // @declaration a constructor of `ShortCircuitFalse`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `ShortCircuitFalse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ShortCircuitFalse.calls = ShortCircuitFalse.calls + 1;
        return true;
    }

    static void assign(boolean left) {
        // @method assign(Z)V
        // @declaration a static method of `ShortCircuitFalse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ShortCircuitFalse.result = (left ? rhs() ? 1 : 0 : 0) % 2 != 0;
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ShortCircuitFalse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        assign(false);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("false-result=").append(ShortCircuitFalse.result).append(",calls=").append(ShortCircuitFalse.calls).toString());
        assign(true);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("true-result=").append(ShortCircuitFalse.result).append(",calls=").append(ShortCircuitFalse.calls).toString());
        return;
    }
}
