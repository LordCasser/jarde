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

    static int run(int x) {
        // @method run(I)I
        // @declaration a static method of `ListThenMap`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList v = new java.util.ArrayList();
        v.add((java.lang.Object) java.lang.Integer.valueOf(x));
        v.add((java.lang.Object) java.lang.Integer.valueOf(x + 1));
        int n = v.size();
        v = new java.util.HashMap();
        v.put((java.lang.Object) "k", (java.lang.Object) java.lang.Integer.valueOf(n));
        return v.size() + ((java.lang.Integer) v.get((java.lang.Object) "k")).intValue();
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ListThenMap`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(run(4));
        return;
    }
}
