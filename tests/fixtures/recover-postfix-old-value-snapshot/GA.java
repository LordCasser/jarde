public class GA<T> {
    private Object[] elems; private int size;                        // 泛型集合内部（Object[] 存储）
    GA(int cap){ elems = new Object[cap]; }                          // 普通 Object 数组
    @SuppressWarnings("unchecked")
    T[] typedArray(int n){ return (T[]) new Object[n]; }             // (T[]) cast 惯用法（堆污染）
    void add(T t){ elems[size++] = t; }
    @SuppressWarnings("unchecked")
    T get(int i){ return (T) elems[i]; }                             // 单元素 cast
    static <U> U[] fill(U u, int n){ U[] arr = (U[]) new Object[n]; for(int i = 0; i < n; i++){ arr[i] = u; } return arr; }  // 静态泛型方法版
    interface Cfg { int LIMIT = compute(); java.util.List<String> NAMES = java.util.Arrays.asList("a", "b");   // 接口字段初始化（接口 clinit）
                   static int compute(){ return 10 * 2; } }
    public static void main(String[] a){ GA<String> g = new GA<>(4); g.add("x"); System.out.println(""+g.get(0)+"/"+g.typedArray(2).length+"/"+java.util.Arrays.toString(fill(7, 3))+"/"+Cfg.LIMIT+"/"+Cfg.NAMES.size()); }
}
