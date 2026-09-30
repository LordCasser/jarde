// jarde: presentation of `C4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C4 extends java.lang.Object implements java.lang.AutoCloseable {
    public C4() {
        // @method <init>()V
        // @declaration a constructor of `C4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `C4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String twrNamed() throws java.lang.Exception {
        // @method twrNamed()Ljava/lang/String;
        // @declaration a static method of `C4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (C4 local0 = new C4()) {
            local0.toString();
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static java.lang.String constructNamed() {
        // @method constructNamed()Ljava/lang/String;
        // @declaration a static method of `C4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        local0 = new java.lang.StringBuilder();
        try {
            local0.append('a');
        } catch (java.lang.IllegalStateException local1) {
            return "caught";
        }
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) twrNamed());
        java.lang.System.out.println((java.lang.String) constructNamed());
        return;
    }
}
