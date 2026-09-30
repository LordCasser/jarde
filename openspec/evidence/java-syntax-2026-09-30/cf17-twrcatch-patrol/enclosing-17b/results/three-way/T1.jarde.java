// jarde: presentation of `T1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T1 extends java.lang.Object implements java.lang.AutoCloseable {
    public T1() {
        // @method <init>()V
        // @declaration a constructor of `T1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `T1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String twrVoid() throws java.lang.Exception {
        // @method twrVoid()Ljava/lang/String;
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T1 local0 = new T1()) {
            local0.hashCode();
        }
        return "done";
    }

    public static java.lang.String twrPop() throws java.lang.Exception {
        // @method twrPop()Ljava/lang/String;
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T1 local0 = new T1()) {
            local0.toString();
        }
        return "done";
    }

    public static java.lang.String twrPopNamed() throws java.lang.Exception {
        // @method twrPopNamed()Ljava/lang/String;
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T1 local0 = new T1()) {
            local0.toString();
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static java.lang.String twrVoidNamed() throws java.lang.Exception {
        // @method twrVoidNamed()Ljava/lang/String;
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T1 local0 = new T1()) {
            local0.hashCode();
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) twrVoid());
        java.lang.System.out.println((java.lang.String) twrPop());
        java.lang.System.out.println((java.lang.String) twrPopNamed());
        java.lang.System.out.println((java.lang.String) twrVoidNamed());
        return;
    }
}
