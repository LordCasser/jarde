// jarde: presentation of `LockGuardProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LockGuardProbe extends java.lang.Object {
    public LockGuardProbe() {
        // @method <init>()V
        // @declaration a constructor of `LockGuardProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int countLines(java.lang.String arg0) throws java.io.IOException {
        // @method countLines(Ljava/lang/String;)I
        // @declaration a static method of `LockGuardProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.io.BufferedReader local1;
        local1 = new java.io.BufferedReader((java.io.Reader) new java.io.InputStreamReader((java.io.InputStream) new java.io.FileInputStream(arg0), "UTF-8"));
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
