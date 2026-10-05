import java.util.BitSet;
public class BT {
    static String permissions(int[] grant, int[] revoke){          // 权限掩码惯用法
        BitSet p = new BitSet();
        for(int g : grant){ p.set(g); }
        for(int r : revoke){ p.clear(r); }
        StringBuilder sb = new StringBuilder();
        for(int i = p.nextSetBit(0); i >= 0; i = p.nextSetBit(i+1)){ sb.append(i).append(','); }
        return sb.length()==0 ? "-" : sb.substring(0, sb.length()-1);
    }
    static int dedup(int[] xs){                                     // BitSet 去重/计数
        BitSet seen = new BitSet();
        for(int x : xs){ seen.set(x); }
        return seen.cardinality();
    }
    static String maskOps(BitSet a, BitSet b){                      // 位集布尔运算链
        BitSet and = (BitSet) a.clone(); and.and(b);
        BitSet or  = (BitSet) a.clone(); or.or(b);
        BitSet xor = (BitSet) a.clone(); xor.xor(b);
        BitSet f = (BitSet) a.clone(); f.flip(0, 8);
        return and.cardinality()+"/"+or.cardinality()+"/"+xor.cardinality()+"/"+f.cardinality();
    }
    public static void main(String[] a){
        BitSet x = new BitSet(); x.set(1); x.set(2); x.set(5);
        BitSet y = new BitSet(); y.set(2); y.set(5); y.set(7);
        System.out.println(permissions(new int[]{1,2,3,5}, new int[]{3}));
        System.out.println(dedup(new int[]{4,4,7,9,9,9}));
        System.out.println(maskOps(x,y));
    }
}
