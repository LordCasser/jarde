// jarde: presentation of `VN1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VN1 extends java.lang.Object {
    static Other$Inner box = new Other$Inner();

    public VN1() {
        // @method <init>()V
        // @declaration a constructor of `VN1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int use(Other$Inner arg0, int arg1) {
        // @method use(LOther$Inner;I)I
        // @declaration a static method of `VN1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Other$Inner local2 = new Other$Inner();
        boolean local3 = (java.lang.Object) local2 instanceof Other$Inner;
        int local4 = ((Other$Inner) local2).id(arg1);
        int local5 = Other$Inner.twice(arg1);
        if (local3) {
            return arg0.id(arg1) + local4;
        } else {
            return local5;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VN1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(use(new Other$Inner(), 21));
        return;
    }
}
