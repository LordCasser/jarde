// jarde: presentation of `demo/Runner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public final class Runner extends java.lang.Object {
    public Runner() {
        // @method <init>()V
        // @declaration a constructor of `demo.Runner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `demo.Runner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.HashSet local1;
        java.lang.reflect.Constructor[] local2;
        int local3;
        int local4;
        local1 = new java.util.HashSet();
        local2 = demo.Numbers.class.getDeclaredConstructors();
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            Object local5 = local2[local4];
            local1.add((java.lang.Object) java.lang.Integer.valueOf(local5.getParameterTypes().length));
        }
        local2 = new java.util.HashSet();
        local2.add((java.lang.Object) java.lang.Integer.valueOf(2));
        local2.add((java.lang.Object) java.lang.Integer.valueOf(3));
        if (demo.Numbers.values().length == 2) {
            if (demo.Numbers.ZERO.getN() == 0) {
                if (demo.Numbers.ONE.getN() == 1) {
                    if (local1.equals((java.lang.Object) local2)) {
                        if (demo.Numbers.ZERO.getDeclaringClass() != demo.Numbers.class) {
                        } else {
                            java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("values=ZERO:").append(demo.Numbers.ZERO.getN()).append(",ONE:").append(demo.Numbers.ONE.getN()).toString());
                            java.lang.System.out.println("declared-constructors=" + local1);
                            return;
                        }
                    }
                }
            }
        }
        throw new java.lang.AssertionError((java.lang.Object) "TestEnums6 behavior differs");
    }
}
