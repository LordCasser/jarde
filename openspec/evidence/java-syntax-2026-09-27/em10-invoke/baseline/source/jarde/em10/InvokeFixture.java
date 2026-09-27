// jarde: presentation of `em10/InvokeFixture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em10;

public class InvokeFixture extends em10.InvokeBase {
    private int receivers;

    public InvokeFixture() {
        // @method <init>()V
        // @declaration a constructor of `em10.InvokeFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private em10.InvokeWorker receiver() {
        // @method receiver()Lem10/InvokeWorker;
        // @declaration an instance method of `em10.InvokeFixture`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.receivers += 1;
        return new em10.InvokeWorker();
    }

    private void fail() throws java.io.IOException {
        // @method fail()V
        // @declaration an instance method of `em10.InvokeFixture`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.io.IOException("io");
    }

    long run() {
        // @method run()J
        // @declaration an instance method of `em10.InvokeFixture`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.receiver().combine(1, 2L, 0x1.8000000000000p1d, 4) + em10.InvokeTools.combine(5, 6L, 0x1.c000000000000p2d, 8) + inherited(9L, 10) + this.runCatch();
    }

    private long runCatch() {
        // @method runCatch()J
        // @declaration an instance method of `em10.InvokeFixture`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        try {
            this.fail();
        } catch (java.io.IOException local1) {
            return this.receiver().combine(11, 12L, 0x1.a000000000000p3d, 14) + (long) em10.InvokeTools.caught((java.lang.String) local1.getMessage()).length();
        }
        return 0L;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `em10.InvokeFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new em10.InvokeFixture().run());
        return;
    }
}
