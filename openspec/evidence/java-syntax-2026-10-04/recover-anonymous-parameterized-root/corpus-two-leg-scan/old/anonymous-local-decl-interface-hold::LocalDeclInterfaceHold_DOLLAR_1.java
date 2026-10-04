// jarde: presentation of `LocalDeclInterfaceHold$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class LocalDeclInterfaceHold$1 extends java.lang.Object implements java.lang.Runnable {
    LocalDeclInterfaceHold$1() {
        // @method <init>()V
        // @declaration a constructor of `LocalDeclInterfaceHold$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void run() {
        // @method run()V
        // @declaration an instance method of `LocalDeclInterfaceHold$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        LocalDeclInterfaceHold.event("run:" + LocalDeclInterfaceHold.access$000("inside"));
        return;
    }
}
