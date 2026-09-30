// jarde: presentation of `Tf2Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf2Probe extends java.lang.Object {
    public static boolean failCleanup;

    public Tf2Probe() {
        // @method <init>()V
        // @declaration a constructor of `Tf2Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public Result test(byte[] arg1) throws java.lang.Exception {
        // @method test([B)LResult;
        // @declaration an instance method of `Tf2Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.InputStream local2;
        local2 = null;
        try {
            local2 = this.getInputStream(arg1);
            this.decode(local2);
            Result local3 = new Result(400);
            return local3;
        } finally {
            this.closeQuietly(local2);
        }
    }

    private java.io.InputStream getInputStream(byte[] arg1) throws java.lang.Exception {
        // @method getInputStream([B)Ljava/io/InputStream;
        // @declaration an instance method of `Tf2Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if (Support.failQuery) {
            return null;
        } else {
            return new java.io.ByteArrayInputStream(arg1);
        }
    }

    private int decode(java.io.InputStream arg1) throws java.lang.Exception {
        // @method decode(Ljava/io/InputStream;)I
        // @declaration an instance method of `Tf2Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return Support.available(arg1);
    }

    private void closeQuietly(java.io.InputStream arg1) {
        // @method closeQuietly(Ljava/io/InputStream;)V
        // @declaration an instance method of `Tf2Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        Support.closes = Support.closes + 1;
        if (Tf2Probe.failCleanup) {
            throw new java.lang.RuntimeException("cleanup");
        } else {
            return;
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Tf2Probe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Tf2Probe.failCleanup = false;
    }
}
