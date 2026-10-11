// jarde: presentation of `cf11/HoistedIteratorPositive` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class HoistedIteratorPositive extends java.lang.Object {
    public static int bodyCalls;

    public HoistedIteratorPositive() {
        // @method <init>()V
        // @declaration a constructor of `cf11.HoistedIteratorPositive`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sumNested(Ljava/util/List;Ljava/util/List;)I`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public static int sumNested(java.util.List outer, java.util.List inner) {
        // @method sumNested(Ljava/util/List;Ljava/util/List;)I
        // @declaration a static method of `cf11.HoistedIteratorPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int total;
        java.util.Iterator local3;
        total = 0;
        local3 = outer.iterator();
        java.lang.String left;
        java.util.Iterator local5;
        while (local3.hasNext()) {
            left = (java.lang.String) local3.next();
            for (java.lang.Object iteratorElement49 : inner) {
                java.lang.String right = (java.lang.String) iteratorElement49;
                total = total + body(left, right);
            }
        }
        return total;
    }

    private static int body(java.lang.String left, java.lang.String right) {
        // @method body(Ljava/lang/String;Ljava/lang/String;)I
        // @declaration a static method of `cf11.HoistedIteratorPositive`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.HoistedIteratorPositive.bodyCalls = cf11.HoistedIteratorPositive.bodyCalls + 1;
        return left.length() + right.length();
    }
}
