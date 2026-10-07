// jarde: presentation of `WideningProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class WideningProbe extends java.lang.Object {
    private WideningProbe() {
        // @method <init>()V
        // @declaration a constructor of `WideningProbe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.io.InputStreamReader wrap(java.lang.String arg0) throws java.io.IOException {
        // @method wrap(Ljava/lang/String;)Ljava/io/InputStreamReader;
        // @declaration a static method of `WideningProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.io.InputStreamReader((java.io.InputStream) new java.io.FileInputStream(arg0), "UTF-8");
    }

    static java.io.BufferedReader buffer(java.io.InputStreamReader arg0) {
        // @method buffer(Ljava/io/InputStreamReader;)Ljava/io/BufferedReader;
        // @declaration a static method of `WideningProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.io.BufferedReader((java.io.Reader) arg0);
    }
}
