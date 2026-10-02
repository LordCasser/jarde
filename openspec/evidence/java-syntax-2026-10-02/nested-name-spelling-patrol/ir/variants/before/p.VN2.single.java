// jarde: presentation of `p/VN2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public class VN2 extends java.lang.Object {
    public VN2() {
        // @method <init>()V
        // @declaration a constructor of `p.VN2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int deep(p.A$B$C arg0, int arg1) {
        // @method deep(Lp/A$B$C;I)I
        // @declaration a static method of `p.VN2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the instruction at BCI 1 is not part of the provable subset
        return p.A$B$C.three(arg1);
    }

    static int own(p.VN2$Mid$Leaf arg0, int arg1) {
        // @method own(Lp/VN2$Mid$Leaf;I)I
        // @declaration a static method of `p.VN2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return p.VN2$Mid$Leaf.leaf(arg1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.VN2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(deep(new p.A$B$C(), 4)).append(":").append(own(new p.VN2$Mid$Leaf(), 4)).toString());
        return;
    }
}
