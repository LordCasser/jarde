// jarde: presentation of `Cf10Values` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Cf10Values extends java.lang.Object {
    public Cf10Values() {
        // @method <init>()V
        // @declaration a constructor of `Cf10Values`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int intSum(int[] data) {
        // @method intSum([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < data.length) {
            sum = sum + data[i];
            try {
                sum = sum + risky(data[i]);
            } catch (java.lang.IllegalStateException e) {
                sum = sum - 1;
            }
            i = i + 2;
        }
        return sum;
    }

    public static double doubleSum(double[] data) {
        // @method doubleSum([D)D
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        double sum;
        int i;
        sum = 0x0.0000000000000p-1022d;
        i = 0;
        while (i < data.length) {
            sum = sum + data[i];
            try {
                noise(data[i]);
            } catch (java.lang.IllegalStateException e) {
                sum = data[1];
            }
            i = i + 2;
        }
        return sum;
    }

    public static java.lang.String lastRef(java.lang.String[] parts) {
        // @method lastRef([Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String cur;
        int i;
        cur = parts[0];
        i = 1;
        while (i < parts.length) {
            try {
                cur = parts[i];
                noise(cur);
            } catch (java.lang.IllegalStateException e) {
                cur = parts[0];
            }
            i = i + 2;
        }
        return cur;
    }

    public static int catchMultiWrite(int[] data) {
        // @method catchMultiWrite([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < data.length) {
            sum = sum + data[i];
            try {
                sum = sum + risky(data[i]);
            } catch (java.lang.IllegalStateException e) {
                sum = data[1];
                sum = sum + data[2];
            }
            i = i + 2;
        }
        return sum;
    }

    public static int twoTries(int[] data) {
        // @method twoTries([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < data.length) {
            sum = sum + data[i];
            try {
                sum = sum + risky(data[i]);
            } catch (java.lang.IllegalStateException e) {
                sum = sum - 1;
            }
            try {
                noise2(sum);
            } catch (java.lang.ArithmeticException e) {
                sum = data[1];
            }
            i = i + 2;
        }
        return sum;
    }

    static int risky(int v) {
        // @method risky(I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 3) {
            throw new java.lang.IllegalStateException("r");
        } else {
            return v;
        }
    }

    static void noise2(int v) {
        // @method noise2(I)V
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 4) {
            throw new java.lang.ArithmeticException("m");
        } else {
            return;
        }
    }

    static void noise(double v) {
        // @method noise(D)V
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 0x1.8000000000000p1d) {
            throw new java.lang.IllegalStateException("n");
        } else {
            return;
        }
    }

    static void noise(java.lang.String v) {
        // @method noise(Ljava/lang/String;)V
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if ("boom".equals((java.lang.Object) v)) {
            throw new java.lang.IllegalStateException("n");
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] d = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(intSum(d));
        double[] f = new double[]{0x1.8000000000000p0d, 0x1.4000000000000p1d, 0x1.8000000000000p1d, 0x1.0000000000000p2d, 0x1.6000000000000p2d};
        java.lang.System.out.println(doubleSum(f));
        java.lang.String[] p = new java.lang.String[]{"a", "boom", "c", "d", "e", "f"};
        java.lang.System.out.println((java.lang.String) lastRef(p));
        java.lang.System.out.println(catchMultiWrite(d));
        java.lang.System.out.println(twoTries(d));
        return;
    }
}
