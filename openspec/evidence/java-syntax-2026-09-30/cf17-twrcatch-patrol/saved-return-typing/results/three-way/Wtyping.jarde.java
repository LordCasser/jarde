public class Wtyping extends java.lang.Object implements java.lang.AutoCloseable {
    public static boolean boom;

    public Wtyping() {
        // @method <init>()V
        // @declaration a constructor of `Wtyping`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `Wtyping`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (Wtyping.boom) {
            throw new java.lang.IllegalStateException("close");
        } else {
            return;
        }
    }

    static void touch(Wtyping arg0) {
        // @method touch(LWtyping;)V
        // @declaration a static method of `Wtyping`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (Wtyping.boom) {
            throw new java.lang.IllegalStateException("touch");
        } else {
            return;
        }
    }

    public static java.lang.String stringLiteral() throws java.lang.Exception {
        // @method stringLiteral()Ljava/lang/String;
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            java.lang.String local1 = "in";
            return local1;
        }
    }

    public static int intLiteral() throws java.lang.Exception {
        // @method intLiteral()I
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            int local1 = 42;
            return local1;
        }
    }

    public static java.lang.StringBuilder constructorValue() throws java.lang.Exception {
        // @method constructorValue()Ljava/lang/StringBuilder;
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            java.lang.StringBuilder local1 = new java.lang.StringBuilder("built");
            return local1;
        }
    }

    public static java.lang.String callReturn() throws java.lang.Exception {
        // @method callReturn()Ljava/lang/String;
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            java.lang.String local1 = java.lang.String.valueOf(7);
            return local1;
        }
    }

    public static java.lang.String nullValue() throws java.lang.Exception {
        // @method nullValue()Ljava/lang/String;
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            Object local1 = null;
            return local1;
        }
    }

    // jarde: generic Signature projection refused for `classLiteral()Ljava/lang/Class;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public static java.lang.Class classLiteral() throws java.lang.Exception {
        // @method classLiteral()Ljava/lang/Class;
        // @declaration a static method of `Wtyping`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (Wtyping local0 = new Wtyping()) {
            touch(local0);
            java.lang.Class local1 = java.lang.String.class;
            return local1;
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Wtyping`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Wtyping.boom = false;
    }
}
