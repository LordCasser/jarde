public class RG {
    static class Hold<T> { T v; Hold(T v){ this.v = v; } T get(){ return v; } }
    static Hold<String> field = new Hold<>("sf");                          // 菱形字段初始化
    static java.util.List<String> names = java.util.Arrays.asList("a","b"); // 泛型静态字段
    static Hold<Hold<Integer>> nested = new Hold<>(new Hold<>(5));          // 嵌套菱形
    <T extends Comparable<T>> T pick(T a, T b){ return a.compareTo(b) >= 0 ? a : b; } // 泛型方法+递归界
    static void consume(){ RG r = new RG(); String s = r.pick("x","y"); Integer i = r.pick(3,7); Hold<String> h = new RG().newInner(); System.out.println(s+i+h.get()+field.get()+nested.get().get()); }
    Hold<String> newInner(){ return new Hold<>("in"); }
    public static void main(String[] a){ consume(); System.out.println(names.size()); }
}
