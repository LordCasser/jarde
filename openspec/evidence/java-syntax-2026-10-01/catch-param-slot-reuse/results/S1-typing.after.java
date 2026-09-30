// jarde: presentation of `S1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class S1 extends java.lang.Object implements java.lang.AutoCloseable {
    public S1() {
        // @method <init>()V
        // @declaration a constructor of `S1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `S1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static java.lang.String tag(java.lang.IllegalStateException arg0) {
        // @method tag(Ljava/lang/IllegalStateException;)Ljava/lang/String;
        // @declaration a static method of `S1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "tag:" + arg0.getMessage();
    }

    static void touch(S1 arg0) {
        // @method touch(LS1;)V
        // @declaration a static method of `S1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String twrHelper() throws java.lang.Exception {
        // @method twrHelper()Ljava/lang/String;
        // @declaration a static method of `S1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (S1 local0 = new S1()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            return tag(local0);
        }
        return "done";
    }

    public static java.lang.String plainHelper() throws java.lang.Exception {
        // @method plainHelper()Ljava/lang/String;
        // @declaration a static method of `S1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            touch((S1) null);
        } catch (java.lang.IllegalStateException local0) {
            return tag(local0);
        }
        return "done";
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `S1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) twrHelper());
        java.lang.System.out.println((java.lang.String) plainHelper());
        return;
    }
}
