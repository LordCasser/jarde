// jarde: presentation of `IOMidRead` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class IOMidRead extends java.lang.Object {
    private IOMidRead() {
        // @method <init>()V
        // @declaration a constructor of `IOMidRead`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int countRemaining(java.io.BufferedReader arg0) throws java.io.IOException {
        // @method countRemaining(Ljava/io/BufferedReader;)I
        // @declaration a static method of `IOMidRead`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.io.BufferedReader local1;
        local1 = arg0;
        try {
            int local2;
            local2 = 0;
            while (local1.readLine() != null) {
                local2 = local2 + 1;
            }
            int local4 = local2;
            return local4;
        } finally {
            local1.close();
        }
    }
}
