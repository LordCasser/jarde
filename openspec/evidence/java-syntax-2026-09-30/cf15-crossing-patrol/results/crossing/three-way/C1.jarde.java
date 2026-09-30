// jarde: presentation of `C1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C1 extends java.lang.Object {
    public C1() {
        // @method <init>()V
        // @declaration a constructor of `C1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int work(int arg0) {
        // @method work(I)I
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + 1;
    }

    public static java.lang.String swallow() {
        // @method swallow()Ljava/lang/String;
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            work(1);
            return "ok";
        } catch (java.lang.NoSuchFieldError local0) {
            return "missed";
        }
    }

    public static java.lang.String five() {
        // @method five()Ljava/lang/String;
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        local0 = new java.lang.StringBuilder();
        try {
            local0.append('a');
        } catch (java.lang.NoSuchFieldError local1) {
        }
        try {
            local0.append('b');
        } catch (java.lang.NoSuchFieldError local1) {
        }
        try {
            local0.append('c');
        } catch (java.lang.IllegalStateException local1) {
        }
        try {
            local0.append('d');
        } catch (java.lang.NoSuchFieldError local1) {
        }
        try {
            local0.append('e');
        } catch (java.lang.RuntimeException local1) {
        }
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) swallow());
        java.lang.System.out.println((java.lang.String) five());
        return;
    }
}
