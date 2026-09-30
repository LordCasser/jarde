public class T2 extends java.lang.Object implements java.lang.AutoCloseable {
    public T2() {
        // @method <init>()V
        // @declaration a constructor of `T2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `T2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static void touch(T2 arg0) {
        // @method touch(LT2;)V
        // @declaration a static method of `T2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String voidBody() throws java.lang.Exception {
        // @method voidBody()Ljava/lang/String;
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T2 local0 = new T2()) {
            touch(local0);
        }
        return "done";
    }

    public static java.lang.String voidBodyReturnInside() throws java.lang.Exception {
        // @method voidBodyReturnInside()Ljava/lang/String;
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T2 local0 = new T2()) {
            touch(local0);
            java.lang.String local1 = "in";
            return local1;
        }
    }

    public static java.lang.String popBody() throws java.lang.Exception {
        // @method popBody()Ljava/lang/String;
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T2 local0 = new T2()) {
            local0.toString();
        }
        return "done";
    }

    public static java.lang.String popBodyVoidTouch() throws java.lang.Exception {
        // @method popBodyVoidTouch()Ljava/lang/String;
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T2 local0 = new T2()) {
            local0.hashCode();
            touch(local0);
        }
        return "done";
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) voidBody());
        java.lang.System.out.println((java.lang.String) voidBodyReturnInside());
        java.lang.System.out.println((java.lang.String) popBody());
        java.lang.System.out.println((java.lang.String) popBodyVoidTouch());
        return;
    }
}
