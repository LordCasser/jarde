import java.util.BitSet;
public class BT extends java.lang.Object {
    static int dedup(int[] arg0) {
        java.util.BitSet local1;
        int[] local2;
        local1 = new java.util.BitSet();
        local2 = arg0;
        for (int local5 : local2) {
            local1.set(local5);
        }
        return local1.cardinality();
    }

    static java.lang.String maskOps(java.util.BitSet arg0, java.util.BitSet arg1) {
        java.util.BitSet local2 = (java.util.BitSet) arg0.clone();
        local2.and(arg1);
        java.util.BitSet local3 = (java.util.BitSet) arg0.clone();
        local3.or(arg1);
        java.util.BitSet local4 = (java.util.BitSet) arg0.clone();
        local4.xor(arg1);
        java.util.BitSet local5 = (java.util.BitSet) arg0.clone();
        local5.flip(0, 8);
        return "" + local2.cardinality() + "/" + local3.cardinality() + "/" + local4.cardinality() + "/" + local5.cardinality();
    }
    public static void main(java.lang.String[] arg0) {
        BitSet x = new BitSet(); x.set(1); x.set(2); x.set(5);
        BitSet y = new BitSet(); y.set(2); y.set(5); y.set(7);
        System.out.println(dedup(new int[]{4,4,7,9,9,9}));
        System.out.println(maskOps(x,y));
        return;
    }
}
