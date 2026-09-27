// jarde: presentation of `em10/InvokeTools` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em10;

class InvokeTools extends java.lang.Object {
    InvokeTools() {
        // @method <init>()V
        // @declaration a constructor of `em10.InvokeTools`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static long combine(int arg0, long arg1, double arg3, int arg5) {
        // @method combine(IJDI)J
        // @declaration a static method of `em10.InvokeTools`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (long) arg0 + arg1 + (long) arg3 + (long) arg5;
    }

    static java.lang.String caught(java.lang.String arg0) {
        // @method caught(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `em10.InvokeTools`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "caught:" + arg0;
    }
}
