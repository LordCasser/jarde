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
        // jarde: not recovered: the recovery run for `wrap(Ljava/lang/String;)Ljava/io/InputStreamReader;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method wrap(Ljava/lang/String;)Ljava/io/InputStreamReader;
        // @declaration a static method of `WideningProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 17
        // the parameter 0 of the invocation at BCI 14 is declared `java.io.InputStream` presents `java.io.FileInputStream` but the invocation requires `java.io.InputStream` and this layer has no safe reference conversion evidence
    }

    static java.io.BufferedReader buffer(java.io.InputStreamReader arg0) {
        // jarde: not recovered: the recovery run for `buffer(Ljava/io/InputStreamReader;)Ljava/io/BufferedReader;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method buffer(Ljava/io/InputStreamReader;)Ljava/io/BufferedReader;
        // @declaration a static method of `WideningProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 8
        // the parameter 0 of the invocation at BCI 5 is declared `java.io.Reader` presents `java.io.InputStreamReader` but the invocation requires `java.io.Reader` and this layer has no safe reference conversion evidence
    }
}
