// jarde: presentation of `VN4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VN4 extends java.lang.Object {
    public VN4() {
        // @method <init>()V
        // @declaration a constructor of `VN4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int area(Other$Box arg0) {
        // @method area(LOther$Box;)I
        // @declaration a static method of `VN4`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size(12);
    }

    static boolean marked(java.lang.Object arg0) {
        // @method marked(Ljava/lang/Object;)Z
        // @declaration a static method of `VN4`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 instanceof Other$Mark;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VN4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Other$Tagged local1 = new Other$Tagged();
        // @bytecode 57 8 54 51 39 34 27 22 19 31 46 43 18 30 42
        // the parameter 0 of the invocation at BCI 19 is declared `Other$Box` presents `Other$Tagged` but the invocation requires `Other$Box` and this layer has no safe reference conversion evidence
        return;
    }
}
