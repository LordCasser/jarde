public class CO {
    static abstract class Node<T extends Comparable<T>> { T val; abstract int cmp(Node<T> o); }
    static class IntNode extends Node<Integer> { IntNode(Integer v){ val = v; } int cmp(Node<Integer> o){ return val.compareTo(o.val); } }
    static <T extends Comparable<T>> T max(T a, T b){ return a.compareTo(b) >= 0 ? a : b; }
    static String callGen(){ return max("a", "b"); }                                        // String→Comparable
    static Integer callGen2(){ return max(1, 2); }                                          // 装箱 Integer→Comparable
    static String same(String a, String b){ return max(a, b); }
    public static void main(String[] a){ System.out.println(""+new IntNode(2).cmp(new IntNode(1))+"/"+callGen()+"/"+callGen2()+"/"+same("a","b")); }
}
