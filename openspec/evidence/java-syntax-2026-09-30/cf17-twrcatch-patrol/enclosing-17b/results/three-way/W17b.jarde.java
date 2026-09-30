// jarde: presentation of `W17b` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W17b extends java.lang.Object implements java.lang.AutoCloseable {
    static boolean closeBoom;

    public W17b() {
        // @method <init>()V
        // @declaration a constructor of `W17b`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `W17b`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (W17b.closeBoom) {
            throw new java.lang.IllegalStateException("close");
        } else {
            return;
        }
    }

    static void touch(W17b arg0) {
        // @method touch(LW17b;)V
        // @declaration a static method of `W17b`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static void boom() {
        // @method boom()V
        // @declaration a static method of `W17b`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.lang.IllegalStateException("body");
    }

    public static java.lang.String normalReturn() {
        // @method normalReturn()Ljava/lang/String;
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17b local0 = new W17b()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static java.lang.String bodyThrows() {
        // @method bodyThrows()Ljava/lang/String;
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17b local0 = new W17b()) {
            boom();
        } catch (java.lang.IllegalStateException local0) {
            return "caught:" + local0.getMessage();
        }
        return "done";
    }

    public static java.lang.String handlerCall() {
        // @method handlerCall()Ljava/lang/String;
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17b local0 = new W17b()) {
            boom();
        } catch (java.lang.IllegalStateException local0) {
            touch((W17b) null);
        }
        return "done";
    }

    public static java.lang.String closeThrows() {
        // @method closeThrows()Ljava/lang/String;
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17b local0 = new W17b()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            return "caught:" + local0.getMessage() + ":" + java.util.Arrays.toString((java.lang.Object[]) local0.getSuppressed());
        }
        return "done";
    }

    public static java.lang.String suppressedBoth() {
        // @method suppressedBoth()Ljava/lang/String;
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17b local0 = new W17b()) {
            boom();
        } catch (java.lang.IllegalStateException local0) {
            return "caught:" + local0.getMessage() + ":" + java.util.Arrays.toString((java.lang.Object[]) local0.getSuppressed());
        }
        return "done";
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W17b`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) normalReturn());
        java.lang.System.out.println((java.lang.String) bodyThrows());
        java.lang.System.out.println((java.lang.String) handlerCall());
        W17b.closeBoom = true;
        java.lang.System.out.println((java.lang.String) closeThrows());
        java.lang.System.out.println((java.lang.String) suppressedBoth());
        W17b.closeBoom = false;
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `W17b`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        W17b.closeBoom = false;
    }
}
