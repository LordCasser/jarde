// jarde: presentation of `Tf3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf3 extends java.lang.Object {
    public byte[] bytes;

    public Tf3() {
        // @method <init>()V
        // @declaration a constructor of `Tf3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public byte[] test() throws java.lang.Exception {
        // @method test()[B
        // @declaration an instance method of `Tf3`, member flags 0x0001
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
        // @declaration an instance method of `Tf3`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return new byte[0];
    }

    private boolean validate() throws java.lang.Exception {
        // @method validate()Z
        // @declaration an instance method of `Tf3`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return false;
    }

    private java.io.InputStream getInputStream() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `getInputStream()Ljava/io/InputStream;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method getInputStream()Ljava/io/InputStream;
        // @declaration an instance method of `Tf3`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10 7 5
        // the copy at BCI 3 has no proved local assignment
    }

    private byte[] read(java.io.InputStream arg1) throws java.lang.Exception {
        // @method read(Ljava/io/InputStream;)[B
        // @declaration an instance method of `Tf3`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return new byte[0];
    }

    private static void close(java.io.InputStream arg0) {
        // @method close(Ljava/io/InputStream;)V
        // @declaration a static method of `Tf3`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
