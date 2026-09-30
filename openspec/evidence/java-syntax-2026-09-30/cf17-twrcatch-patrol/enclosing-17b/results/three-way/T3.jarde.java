// jarde: presentation of `T3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T3 extends java.lang.Object implements java.lang.AutoCloseable {
    public T3() {
        // @method <init>()V
        // @declaration a constructor of `T3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `T3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static void touch(T3 arg0) {
        // @method touch(LT3;)V
        // @declaration a static method of `T3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String voidNamed() throws java.lang.Exception {
        // @method voidNamed()Ljava/lang/String;
        // @declaration a static method of `T3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T3 local0 = new T3()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static java.lang.String voidNamedRecover() throws java.lang.Exception {
        // @method voidNamedRecover()Ljava/lang/String;
        // @declaration a static method of `T3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T3 local0 = new T3()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            touch((T3) null);
        }
        return "done";
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) voidNamed());
        java.lang.System.out.println((java.lang.String) voidNamedRecover());
        return;
    }
}
