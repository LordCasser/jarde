// jarde: presentation of `ListThenMap` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ListThenMap extends java.lang.Object {
    public ListThenMap() {
        // @method <init>()V
        // @declaration a constructor of `ListThenMap`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int run(int arg0) {
        // @method run(I)I
        // @declaration a static method of `ListThenMap`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList local2 = new java.util.ArrayList();
        local2.add((java.lang.Object) java.lang.Integer.valueOf(arg0));
        local2.add((java.lang.Object) java.lang.Integer.valueOf(arg0 + 1));
        int local1 = local2.size();
        java.util.HashMap local2_2 = new java.util.HashMap();
        local2_2.put((java.lang.Object) "k", (java.lang.Object) java.lang.Integer.valueOf(local1));
        return local2_2.size() + ((java.lang.Integer) local2_2.get((java.lang.Object) "k")).intValue();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ListThenMap`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(run(4));
        return;
    }
}
