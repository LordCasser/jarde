// jarde: presentation of `ParentCarrier` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class ParentCarrier extends java.lang.Object {
    ParentCarrier() {
        // @method <init>()V
        // @declaration a constructor of `ParentCarrier`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static class Holder extends java.lang.Object {
        Holder() {
            // @method <init>()V
            // @declaration a constructor of `ParentCarrier$Holder`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        java.lang.String render() {
            // @method render()Ljava/lang/String;
            // @declaration an instance method of `ParentCarrier$Holder`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return "holder";
        }
    }
}
