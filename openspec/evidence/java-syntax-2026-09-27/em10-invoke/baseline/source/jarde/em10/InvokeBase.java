// jarde: presentation of `em10/InvokeBase` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em10;

class InvokeBase extends java.lang.Object {
    InvokeBase() {
        // @method <init>()V
        // @declaration a constructor of `em10.InvokeBase`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static long inherited(long arg0, int arg2) {
        // @method inherited(JI)J
        // @declaration a static method of `em10.InvokeBase`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 * 10L + (long) arg2;
    }
}
