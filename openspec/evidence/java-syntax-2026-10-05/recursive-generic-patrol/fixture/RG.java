public class RG {
    static abstract class Node<T extends Comparable<T>> {          // 递归界
        T val;
        abstract int cmp(Node<T> o);
    }
    static class IntNode extends Node<Integer> {                    // 具体化
        int cmp(Node<Integer> o){ return val.compareTo(o.val); }
    }
    static <T extends Comparable<T>> T max(T a, T b){ return a.compareTo(b) >= 0 ? a : b; }  // 泛型方法
    static String callGen(){ return max("a","b"); }                 // 调用点推断 T=String
    static Integer callGen2(){ return max(1, 2); }                  // 调用点推断 T=Integer（装箱）
    static java.util.List<String> callList(){ return java.util.Collections.singletonList("x"); } // 泛型工厂消费
    public static void main(String[] a){ System.out.println(""+new IntNode().cmp(new IntNode())+"/"+callGen()+"/"+callGen2()+"/"+callList().get(0)); }
}
