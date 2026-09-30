// jarde: presentation of `N1x` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N1x extends java.lang.Object {
    public N1x() {
        // @method <init>()V
        // @declaration a constructor of `N1x`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.io.ByteArrayOutputStream open() {
        // @method open()Ljava/io/ByteArrayOutputStream;
        // @declaration a static method of `N1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.io.ByteArrayOutputStream();
    }

    public static void t(int arg0) {
        // @method t(I)V
        // @declaration a static method of `N1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            throw new java.lang.IllegalStateException("state");
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        run(arg0.length);
        return;
    }

    public static void run(int arg0) {
        // @method run(I)V
        // @declaration a static method of `N1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.ByteArrayOutputStream local1;
        local1 = open();
        try {
            t(arg0);
            local1.write(1);
        } catch (java.lang.IllegalStateException local2) {
            java.lang.System.out.println(local1.size());
        }
        java.lang.System.out.println("end:" + local1.size());
        return;
    }
}
