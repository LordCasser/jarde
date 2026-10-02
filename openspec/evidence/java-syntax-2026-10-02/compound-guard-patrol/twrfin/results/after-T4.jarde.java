// jarde: presentation of `T4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T4 extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    java.lang.String name;

    T4(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `T4`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.name = arg1;
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `T4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        T4.log.append('[').append(this.name).append(']');
        return;
    }

    public static java.lang.String nested() throws java.lang.Exception {
        // @method nested()Ljava/lang/String;
        // @declaration a static method of `T4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (T4 local0 = new T4("a")) {
            try (T4 local1 = new T4("b")) {
                T4.log.append("body");
            } finally {
                T4.log.append("mid");
            }
        }
        return T4.log.toString();
    }

    public static java.lang.Object pick(java.lang.Object arg0) {
        // @method pick(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `T4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 instanceof java.lang.String) {
            return java.lang.Integer.valueOf(((java.lang.String) arg0).length());
        } else if (arg0 instanceof java.lang.Integer) {
            return java.lang.Integer.valueOf(((java.lang.Integer) arg0).intValue() + 1);
    } else {
            return null;
    }
    }

    public static int sync(int arg0) {
        // @method sync(I)I
        // @declaration a static method of `T4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (T4.class) {
            int local2;
            int local4;
            local2 = 0;
            synchronized (T4.log) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    local2 = local2 + local4;
                }
            }
            return local2;
        }
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) nested());
        java.lang.System.out.println("" + pick((java.lang.Object) "hey") + ":" + pick((java.lang.Object) java.lang.Integer.valueOf(41)) + ":" + pick((java.lang.Object) java.lang.Double.valueOf(0x1.0000000000000p2d)));
        java.lang.System.out.println(sync(5));
        return;
    }
}
