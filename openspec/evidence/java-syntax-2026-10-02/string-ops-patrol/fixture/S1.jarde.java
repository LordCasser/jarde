// jarde: presentation of `S1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class S1 extends java.lang.Object {
    public S1() {
        // @method <init>()V
        // @declaration a constructor of `S1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String ops(java.lang.String arg0) {
        // @method ops(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `S1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        local1.append(arg0.charAt(0));
        local1.append((java.lang.String) arg0.substring(1, 3));
        local1.append(arg0.indexOf(97));
        local1.append(arg0.length());
        local1.append((java.lang.String) arg0.toUpperCase());
        local1.append((java.lang.String) java.lang.String.valueOf(42));
        local1.append((java.lang.String) java.lang.String.format("%s-%d", new java.lang.Object[]{arg0, java.lang.Integer.valueOf(7)}));
        java.lang.String local2 = arg0 + "x" + 9;
        local1.append((java.lang.String) local2.replace('a', 'b'));
        local1.append(local2.equals((java.lang.Object) arg0));
        // @bytecode 156 159 142
        // the parameter 0 of the invocation at BCI 156 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `S1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ops("abcd"));
        java.lang.System.out.println((java.lang.String) ops("banana"));
        return;
    }
}
