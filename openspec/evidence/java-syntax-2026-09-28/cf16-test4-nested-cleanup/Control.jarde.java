// jarde: presentation of `jadx/tests/integration/trycatch/Control` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class Control extends java.lang.Object {
    public Control() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.Control`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void run(java.io.OutputStream outputStream, java.io.File file) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `run(Ljava/io/OutputStream;Ljava/io/File;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run(Ljava/io/OutputStream;Ljava/io/File;)V
        // @declaration a static method of `jadx.tests.integration.trycatch.Control`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 5 6 9 10 13 14
        // BCI 2: the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect
        // @bytecode 17 18 21 22 23 26 27 30 31 34 36 37 38
        // 5 live block(s) are reachable only through edges the normal-flow view leaves out: [38, 17, 21, 36, 34]
    }
}
