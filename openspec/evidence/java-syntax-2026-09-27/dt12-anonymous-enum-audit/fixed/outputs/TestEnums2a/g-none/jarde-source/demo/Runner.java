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
        if ("*".equals((java.lang.Object) demo.DoubleOperations.TIMES.getOp())) {
            if ("/".equals((java.lang.Object) demo.DoubleOperations.DIVIDE.getOp())) {
                if (demo.DoubleOperations.TIMES.apply(0x1.0000000000000p1d, 0x1.8000000000000p1d) == 0x1.8000000000000p2d) {
                    if (demo.DoubleOperations.DIVIDE.apply(0x1.4000000000000p3d, 0x1.4000000000000p2d) == 0x1.0000000000000p1d) {
                        if (demo.DoubleOperations.TIMES.getDeclaringClass() == demo.DoubleOperations.class) {
                            if (demo.DoubleOperations.DIVIDE.getDeclaringClass() == demo.DoubleOperations.class) {
                                if (demo.DoubleOperations.TIMES.getClass() != demo.DoubleOperations.class) {
                                    if (demo.DoubleOperations.DIVIDE.getClass() == demo.DoubleOperations.class) {
                                    } else {
                                        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("TIMES=*:6:").append((java.lang.String) demo.DoubleOperations.TIMES.getClass().getName()).toString());
                                        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("DIVIDE=/:2:").append((java.lang.String) demo.DoubleOperations.DIVIDE.getClass().getName()).toString());
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        throw new java.lang.AssertionError((java.lang.Object) "TestEnums2a behavior differs");
    }
}
