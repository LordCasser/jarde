// jarde: presentation of `Tf2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf2 extends java.lang.Object {
    public Tf2() {
        // @method <init>()V
        // @declaration a constructor of `Tf2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private java.io.InputStream getInputStream(byte[] arg1) throws java.io.IOException {
        // @method getInputStream([B)Ljava/io/InputStream;
        // @declaration an instance method of `Tf2`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return new java.io.ByteArrayInputStream(arg1);
    }

    private int decode(java.io.InputStream arg1) throws java.io.IOException {
        // @method decode(Ljava/io/InputStream;)I
        // @declaration an instance method of `Tf2`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return arg1.available();
    }

    private void closeQuietly(java.io.InputStream arg1) {
        // @method closeQuietly(Ljava/io/InputStream;)V
        // @declaration an instance method of `Tf2`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public Tf2$Result test(byte[] arg1) throws java.io.IOException {
        // @method test([B)LTf2$Result;
        // @declaration an instance method of `Tf2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.InputStream local2;
        local2 = null;
        try {
            local2 = this.getInputStream(arg1);
            this.decode(local2);
            Tf2$Result local3 = new Tf2$Result(400);
            return local3;
        } finally {
            this.closeQuietly(local2);
        }
    }
}
