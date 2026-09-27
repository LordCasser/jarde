// jarde: presentation of `cf08/NotIndexedLoop` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08;

public final class NotIndexedLoop extends java.lang.Object {
    public NotIndexedLoop() {
        // @method <init>()V
        // @declaration a constructor of `cf08.NotIndexedLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.io.File test(java.io.File[] arg1) {
        // jarde: not recovered: the recovery run for `test([Ljava/io/File;)Ljava/io/File;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test([Ljava/io/File;)Ljava/io/File;
        // @declaration an instance method of `cf08.NotIndexedLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 11 16 19 25 38 55 58 64 67 69 73 77
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
