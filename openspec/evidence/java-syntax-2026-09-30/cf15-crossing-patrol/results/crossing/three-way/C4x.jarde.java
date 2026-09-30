// jarde: presentation of `C4x` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C4x extends java.lang.Object implements java.lang.AutoCloseable {
    public C4x() {
        // @method <init>()V
        // @declaration a constructor of `C4x`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `C4x`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static void is(int arg0) {
        // @method is(I)V
        // @declaration a static method of `C4x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            throw new java.lang.IllegalStateException("injected");
        } else {
            return;
        }
    }

    public static java.lang.String constructNamed(int arg0) {
        // @method constructNamed(I)Ljava/lang/String;
        // @declaration a static method of `C4x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        try {
            is(arg0);
            local1.append('a');
        } catch (java.lang.IllegalStateException local2) {
            return "caught";
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C4x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) constructNamed(arg0.length));
        return;
    }
}
