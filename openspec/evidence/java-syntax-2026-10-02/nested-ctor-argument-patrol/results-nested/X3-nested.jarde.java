// jarde: presentation of `X3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class X3 extends java.lang.Object {
    public X3() {
        // @method <init>()V
        // @declaration a constructor of `X3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String doubleNested() {
        // @method doubleNested()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf((java.lang.Object) new X3$TwoNested(new X3$B("y"), new X3$C("z")));
    }

    public static java.lang.String secondPosition() {
        // @method secondPosition()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf((java.lang.Object) new X3$Tagged("first", new X3$B("second")));
    }

    public static java.lang.String sameClassTwice() {
        // @method sameClassTwice()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf((java.lang.Object) new X3$TwoSame(new X3$B("1"), new X3$B("2")));
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) doubleNested());
        java.lang.System.out.println((java.lang.String) secondPosition());
        java.lang.System.out.println((java.lang.String) sameClassTwice());
        return;
    }
}
