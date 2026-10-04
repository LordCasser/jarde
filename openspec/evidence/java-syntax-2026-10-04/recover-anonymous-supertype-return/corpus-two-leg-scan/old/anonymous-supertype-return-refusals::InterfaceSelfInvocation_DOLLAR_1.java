// jarde: presentation of `InterfaceSelfInvocation$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class InterfaceSelfInvocation$1 extends java.lang.Object implements Renderer {
    InterfaceSelfInvocation$1() {
        // @method <init>()V
        // @declaration a constructor of `InterfaceSelfInvocation$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String tag() {
        // @method tag()Ljava/lang/String;
        // @declaration an instance method of `InterfaceSelfInvocation$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "tagged";
    }

    public java.lang.String toString() {
        // @method toString()Ljava/lang/String;
        // @declaration an instance method of `InterfaceSelfInvocation$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.tag();
    }
}
