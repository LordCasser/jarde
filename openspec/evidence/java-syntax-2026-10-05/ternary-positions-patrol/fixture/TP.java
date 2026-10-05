public class TP {
    static String pick(String a, String b){ return (a != null ? a : b).trim(); }        // 接收者位三元
    static int idx(int[] xs){ return xs[1 > 0 ? 0 : 1]; }                                // 下标位三元
    static int chain(String a, StringBuilder sb){ return (a != null ? sb.append(a) : sb).length(); }   // 接收者位异型（SB append 返回 SB）
    static String nested(String s){ return ((s != null ? s : "x").isEmpty() ? "E" : "N"); }   // 嵌套条件消费
    public static void main(String[] a){ System.out.println(""+pick(null," q ")+"/"+idx(new int[]{7,9})+"/"+chain("z", new StringBuilder())+"/"+nested("")+"/"+nested(null)); }
}
