// jarde: presentation of `C1x` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C1x extends java.lang.Object {
    public C1x() {
        // @method <init>()V
        // @declaration a constructor of `C1x`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void ns(int arg0) {
        // @method ns(I)V
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            throw new java.lang.NoSuchFieldError("injected");
        } else {
            return;
        }
    }

    public static void is(int arg0) {
        // @method is(I)V
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            throw new java.lang.IllegalStateException("injected");
        } else {
            return;
        }
    }

    public static void re(int arg0) {
        // @method re(I)V
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            throw new java.lang.RuntimeException("injected");
        } else {
            return;
        }
    }

    public static int work(int arg0) {
        // @method work(I)I
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + 1;
    }

    public static java.lang.String swallow(int arg0) {
        // @method swallow(I)Ljava/lang/String;
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            ns(arg0);
            work(1);
            return "ok";
        } catch (java.lang.NoSuchFieldError local1) {
            return "missed";
        }
    }

    public static java.lang.String five(int arg0) {
        // @method five(I)Ljava/lang/String;
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        try {
            ns(arg0);
            local1.append('a');
        } catch (java.lang.NoSuchFieldError local2) {
        }
        try {
            local1.append('b');
        } catch (java.lang.NoSuchFieldError local2) {
        }
        try {
            is(arg0);
            local1.append('c');
        } catch (java.lang.IllegalStateException local2) {
        }
        try {
            local1.append('d');
        } catch (java.lang.NoSuchFieldError local2) {
        }
        try {
            re(arg0);
            local1.append('e');
        } catch (java.lang.RuntimeException local2) {
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) swallow(arg0.length));
        java.lang.System.out.println((java.lang.String) five(arg0.length));
        return;
    }
}
