// jarde: presentation of `Tf3Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf3Probe extends java.lang.Object {
    public static boolean failCleanup;

    public byte[] bytes;

    public Tf3Probe() {
        // @method <init>()V
        // @declaration a constructor of `Tf3Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public byte[] test() throws java.lang.Exception {
        // @method test()[B
        // @declaration an instance method of `Tf3Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.InputStream local1;
        local1 = null;
        try {
            byte[] local2;
            if (this.bytes == null) {
                if (!this.validate()) {
                    return null;
                } else {
                    local1 = this.getInputStream();
                    this.bytes = this.read(local1);
                }
            }
            local2 = this.convert(this.bytes);
            return local2;
        } finally {
            close(local1);
        }
    }

    private byte[] convert(byte[] arg1) throws java.lang.Exception {
        // @method convert([B)[B
        // @declaration an instance method of `Tf3Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    private boolean validate() throws java.lang.Exception {
        // @method validate()Z
        // @declaration an instance method of `Tf3Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return !Support.failValidate;
    }

    private java.io.InputStream getInputStream() throws java.lang.Exception {
        // @method getInputStream()Ljava/io/InputStream;
        // @declaration an instance method of `Tf3Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if (Support.failBody) {
            throw new java.lang.RuntimeException("body");
        } else {
            return new java.io.ByteArrayInputStream((byte[]) Support.preset());
        }
    }

    private byte[] read(java.io.InputStream arg1) throws java.lang.Exception {
        // @method read(Ljava/io/InputStream;)[B
        // @declaration an instance method of `Tf3Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return Support.read(arg1);
    }

    private static void close(java.io.InputStream arg0) {
        // @method close(Ljava/io/InputStream;)V
        // @declaration a static method of `Tf3Probe`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        Support.closes = Support.closes + 1;
        if (Tf3Probe.failCleanup) {
            throw new java.lang.RuntimeException("cleanup");
        } else {
            return;
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Tf3Probe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Tf3Probe.failCleanup = false;
    }
}
