// jarde: presentation of `em12/Parent` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em12;

class Parent extends java.lang.Object {
    Parent() {
        // @method <init>()V
        // @declaration a constructor of `em12.Parent`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    java.lang.String pick(em12.Arg arg1) {
        // @method pick(Lem12/Arg;)Ljava/lang/String;
        // @declaration an instance method of `em12.Parent`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return "number";
    }

    java.lang.String pick(em12.NarrowArg arg1) {
        // @method pick(Lem12/NarrowArg;)Ljava/lang/String;
        // @declaration an instance method of `em12.Parent`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return "integer";
    }
}
